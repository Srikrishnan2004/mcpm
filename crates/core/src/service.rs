//! High-level operations used by both the desktop app and the CLI.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::adapters::{self, ClientInfo};
use crate::catalog::Catalog;
use crate::model::{
    instance_id, is_valid_slug, Binding, ClientId, Instance, Project, Template, TransportKind,
};
use crate::render::{self, LaunchSpec};
use crate::secrets::SecretStore;
use crate::store::{DataDir, Projects, Registry, PROJECTS, REGISTRY};
use crate::sync::{self, SyncReport};
use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInstance {
    pub template: String,
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub transport: Option<TransportKind>,
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
    #[serde(default)]
    pub secrets: BTreeMap<String, String>,
}

pub struct Mcpm {
    data: DataDir,
    secrets: Box<dyn SecretStore>,
    catalog: Catalog,
}

impl Mcpm {
    pub fn open(data: DataDir, secrets: Box<dyn SecretStore>) -> Result<Self> {
        let catalog = Catalog::load(&data)?;
        Ok(Self {
            data,
            secrets,
            catalog,
        })
    }

    pub fn data(&self) -> &DataDir {
        &self.data
    }

    // ---- templates & clients -------------------------------------------------------------

    pub fn templates(&self) -> Vec<Template> {
        self.catalog.all()
    }

    pub fn clients(&self) -> Vec<ClientInfo> {
        adapters::all()
            .iter()
            .map(|a| adapters::info(a.as_ref()))
            .collect()
    }

    // ---- instances -----------------------------------------------------------------------

    pub fn instances(&self) -> Result<Vec<Instance>> {
        Ok(self.data.load::<Registry>(REGISTRY)?.instances)
    }

    pub fn instance(&self, id: &str) -> Result<Instance> {
        self.instances()?
            .into_iter()
            .find(|i| i.id == id)
            .ok_or_else(|| Error::NotFound(format!("instance `{id}`")))
    }

    pub fn create_instance(&self, input: NewInstance) -> Result<Instance> {
        let template = self.catalog.get(&input.template)?;
        let id = instance_id(&template.id, &input.name)?;
        let mut registry: Registry = self.data.load(REGISTRY)?;
        if registry.instances.iter().any(|i| i.id == id) {
            return Err(Error::Invalid(format!("instance `{id}` already exists")));
        }

        let transport = input.transport.unwrap_or(template.default_transport);
        render::effective_settings(template, &input.settings)?;

        let mut stored = Vec::new();
        for spec in &template.secrets {
            match input
                .secrets
                .get(&spec.key)
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
            {
                Some(value) => {
                    self.secrets.set(&id, &spec.key, value)?;
                    stored.push(spec.key.clone());
                }
                None if spec.required => {
                    return Err(Error::Invalid(format!(
                        "missing required secret `{}`",
                        spec.key
                    )))
                }
                None => {}
            }
        }

        let instance = Instance {
            id,
            template: template.id.clone(),
            label: input.label,
            transport,
            settings: input.settings,
            secrets: stored,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        registry.instances.push(instance.clone());
        self.data.save(REGISTRY, &registry)?;
        Ok(instance)
    }

    pub fn delete_instance(&self, id: &str) -> Result<()> {
        if let Some(p) = self
            .projects()?
            .into_iter()
            .find(|p| p.bindings.iter().any(|b| b.instance == id))
        {
            return Err(Error::Invalid(format!(
                "`{id}` is bound in project `{}`; remap or remove that binding first",
                p.name
            )));
        }
        let mut registry: Registry = self.data.load(REGISTRY)?;
        let Some(pos) = registry.instances.iter().position(|i| i.id == id) else {
            return Err(Error::NotFound(format!("instance `{id}`")));
        };
        let instance = registry.instances.remove(pos);
        for key in &instance.secrets {
            self.secrets.delete(id, key)?;
        }
        self.data.save(REGISTRY, &registry)
    }

    pub fn set_secret(&self, id: &str, key: &str, value: &str) -> Result<()> {
        let mut registry: Registry = self.data.load(REGISTRY)?;
        let instance = registry
            .instances
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| Error::NotFound(format!("instance `{id}`")))?;
        let template = self.catalog.get(&instance.template)?;
        if !template.secrets.iter().any(|s| s.key == key) {
            return Err(Error::Invalid(format!(
                "template `{}` has no secret `{key}`",
                template.id
            )));
        }
        self.secrets.set(id, key, value)?;
        if !instance.secrets.iter().any(|k| k == key) {
            instance.secrets.push(key.to_string());
        }
        self.data.save(REGISTRY, &registry)
    }

    /// Resolve an instance into the concrete command/url, reading secrets from the keychain.
    pub fn launch_spec(&self, id: &str) -> Result<LaunchSpec> {
        let instance = self.instance(id)?;
        let template = self.catalog.get(&instance.template)?;
        let secret = |key: &str| {
            self.secrets.get(id, key)?.ok_or_else(|| {
                Error::NotFound(format!(
                    "secret `{key}` for `{id}` (run `mcpm secret-set {id} {key}`)"
                ))
            })
        };
        render::launch_spec(template, &instance, &secret)
    }

    // ---- projects & bindings -------------------------------------------------------------

    pub fn projects(&self) -> Result<Vec<Project>> {
        Ok(self.data.load::<Projects>(PROJECTS)?.projects)
    }

    pub fn add_project(&self, path: &str) -> Result<Project> {
        let dir = Path::new(path);
        if !dir.is_dir() {
            return Err(Error::Invalid(format!("`{path}` is not a folder")));
        }
        let mut projects: Projects = self.data.load(PROJECTS)?;
        if let Some(existing) = projects.projects.iter().find(|p| Path::new(&p.path) == dir) {
            return Ok(existing.clone());
        }
        let project = Project {
            id: format!(
                "p_{:x}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ),
            path: path.to_string(),
            name: dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string()),
            bindings: Vec::new(),
        };
        projects.projects.push(project.clone());
        self.data.save(PROJECTS, &projects)?;
        Ok(project)
    }

    pub fn find_project_by_path(&self, path: &Path) -> Result<Project> {
        self.projects()?
            .into_iter()
            .find(|p| Path::new(&p.path) == path)
            .ok_or_else(|| Error::NotFound(format!("project at {}", path.display())))
    }

    /// Unbinds everything (removing mcpm's entries from the project's files) and forgets the project.
    pub fn remove_project(&self, id: &str) -> Result<SyncReport> {
        let report = self.update_project(id, |p| {
            p.bindings.clear();
            Ok(())
        })?;
        let mut projects: Projects = self.data.load(PROJECTS)?;
        projects.projects.retain(|p| p.id != id);
        self.data.save(PROJECTS, &projects)?;
        Ok(report)
    }

    /// Add a binding or remap an existing one.
    pub fn set_binding(
        &self,
        project_id: &str,
        entry: &str,
        instance: &str,
        clients: Vec<ClientId>,
    ) -> Result<SyncReport> {
        if !is_valid_slug(entry) {
            return Err(Error::Invalid(format!(
                "invalid entry name `{entry}`: use lowercase letters, digits and dashes"
            )));
        }
        if clients.is_empty() {
            return Err(Error::Invalid("select at least one client".into()));
        }
        self.instance(instance)?;
        self.update_project(project_id, |p| {
            let binding = Binding {
                entry: entry.to_string(),
                instance: instance.to_string(),
                clients,
            };
            match p.bindings.iter_mut().find(|b| b.entry == entry) {
                Some(existing) => *existing = binding,
                None => p.bindings.push(binding),
            }
            Ok(())
        })
    }

    pub fn remove_binding(&self, project_id: &str, entry: &str) -> Result<SyncReport> {
        self.update_project(project_id, |p| {
            p.bindings.retain(|b| b.entry != entry);
            Ok(())
        })
    }

    pub fn sync(&self, project_id: &str) -> Result<SyncReport> {
        self.update_project(project_id, |_| Ok(()))
    }

    /// Apply `change` to a project, write client files, and only then persist the project.
    fn update_project(
        &self,
        id: &str,
        change: impl FnOnce(&mut Project) -> Result<()>,
    ) -> Result<SyncReport> {
        let mut projects: Projects = self.data.load(PROJECTS)?;
        let project = projects
            .projects
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| Error::NotFound(format!("project `{id}`")))?;
        change(project)?;
        let report = sync::sync_project(&self.data, project)?;
        self.data.save(PROJECTS, &projects)?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::MemoryStore;
    use serde_json::json;

    #[test]
    fn end_to_end_two_supabase_instances() {
        let tmp = tempfile::tempdir().unwrap();
        let mcpm = Mcpm::open(
            DataDir::at(tmp.path().join("data")),
            Box::new(MemoryStore::default()),
        )
        .unwrap();

        for (name, token) in [("client-a", "tok-a"), ("personal", "tok-p")] {
            mcpm.create_instance(NewInstance {
                template: "supabase".into(),
                name: name.into(),
                label: String::new(),
                transport: None,
                settings: serde_json::from_value(json!({ "project_ref": name })).unwrap(),
                secrets: [("access_token".to_string(), token.to_string())].into(),
            })
            .unwrap();
        }

        let root = tmp.path().join("shop");
        std::fs::create_dir_all(&root).unwrap();
        let project = mcpm.add_project(root.to_str().unwrap()).unwrap();

        mcpm.set_binding(
            &project.id,
            "supabase",
            "supabase@client-a",
            vec![ClientId::ClaudeCode],
        )
        .unwrap();
        mcpm.set_binding(
            &project.id,
            "supabase",
            "supabase@personal",
            vec![ClientId::ClaudeCode],
        )
        .unwrap();
        let text = std::fs::read_to_string(root.join(".mcp.json")).unwrap();
        assert!(text.contains("supabase@personal") && !text.contains("tok-"));

        let LaunchSpec::Stdio { env, .. } = mcpm.launch_spec("supabase@personal").unwrap() else {
            panic!("expected stdio")
        };
        assert_eq!(env["SUPABASE_ACCESS_TOKEN"], "tok-p");

        assert!(mcpm.delete_instance("supabase@personal").is_err());
        mcpm.delete_instance("supabase@client-a").unwrap();
    }
}
