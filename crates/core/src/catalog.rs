//! Server templates: built-ins from `catalog/servers/*.json` plus user templates
//! from `<data>/templates/*.json` (user templates override built-ins with the same id).

use std::collections::BTreeMap;
use std::fs;

use crate::model::Template;
use crate::store::DataDir;
use crate::{Error, Result};

/// Expands file names under `catalog/servers/` into `(name, contents)` pairs baked into the binary.
macro_rules! builtin {
    ($($file:literal),* $(,)?) => {
        &[$(($file, include_str!(concat!("../../../catalog/servers/", $file)))),*]
    };
}

const BUILTIN: &[(&str, &str)] = builtin![
    "airtable.json",
    "atlassian.json",
    "aws-documentation.json",
    "brave-search.json",
    "chrome-devtools.json",
    "cloudflare.json",
    "context7.json",
    "docker.json",
    "exa.json",
    "fetch.json",
    "figma.json",
    "filesystem.json",
    "firebase.json",
    "firecrawl.json",
    "git.json",
    "github.json",
    "gitlab.json",
    "grafana.json",
    "heroku.json",
    "hugging-face.json",
    "linear.json",
    "memory.json",
    "mongodb.json",
    "neon.json",
    "netlify.json",
    "notion.json",
    "perplexity.json",
    "playwright.json",
    "postgres.json",
    "redis.json",
    "sentry.json",
    "sequential-thinking.json",
    "slack.json",
    "stripe.json",
    "supabase.json",
    "tavily.json",
    "time.json",
    "vercel.json",
];

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    templates: BTreeMap<String, Template>,
}

impl Catalog {
    pub fn builtin() -> Result<Self> {
        let mut templates = BTreeMap::new();
        for (name, text) in BUILTIN {
            let t: Template = serde_json::from_str(text).map_err(|e| Error::json(*name, e))?;
            templates.insert(t.id.clone(), t);
        }
        Ok(Self { templates })
    }

    pub fn load(data: &DataDir) -> Result<Self> {
        let mut catalog = Self::builtin()?;
        let dir = data.path("templates");
        let Ok(entries) = fs::read_dir(&dir) else {
            return Ok(catalog);
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                let text = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
                let t: Template = serde_json::from_str(&text).map_err(|e| Error::json(&path, e))?;
                catalog.templates.insert(t.id.clone(), t);
            }
        }
        Ok(catalog)
    }

    pub fn get(&self, id: &str) -> Result<&Template> {
        self.templates
            .get(id)
            .ok_or_else(|| Error::NotFound(format!("template `{id}`")))
    }

    pub fn all(&self) -> Vec<Template> {
        self.templates.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::Value;

    use crate::model::{Instance, TransportKind};
    use crate::render::{launch_spec, LaunchSpec};

    #[test]
    fn builtins_parse() {
        let c = super::Catalog::builtin().unwrap();
        assert!(c.get("supabase").is_ok());
    }

    #[test]
    fn every_catalog_file_is_registered() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog/servers");
        let mut on_disk: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".json"))
            .collect();
        on_disk.sort();
        let registered: Vec<String> = super::BUILTIN.iter().map(|(n, _)| n.to_string()).collect();
        assert_eq!(on_disk, registered, "add new templates to BUILTIN");
    }

    #[test]
    fn every_builtin_renders_its_default_transport() {
        for (file, _) in super::BUILTIN {
            let catalog = super::Catalog::builtin().unwrap();
            let id = file.trim_end_matches(".json");
            let t = catalog
                .get(id)
                .unwrap_or_else(|_| panic!("{file}: id must match file name"));
            let settings: BTreeMap<String, Value> = t
                .settings
                .iter()
                .filter(|s| s.required && s.default.is_none())
                .map(|s| (s.key.clone(), Value::String("x".into())))
                .collect();
            let instance = Instance {
                id: format!("{id}@test"),
                template: id.into(),
                label: String::new(),
                transport: t.default_transport,
                settings,
                secrets: t.secrets.iter().map(|s| s.key.clone()).collect(),
                created_at: String::new(),
            };
            assert_eq!(
                t.default_transport,
                TransportKind::Stdio,
                "{file}: the shim only runs stdio"
            );
            let spec = launch_spec(t, &instance, &|k| Ok(format!("<{k}>")))
                .unwrap_or_else(|e| panic!("{file}: {e}"));
            let LaunchSpec::Stdio { command, args, env } = spec else {
                panic!("{file}: expected stdio")
            };
            for s in std::iter::once(&command).chain(&args).chain(env.values()) {
                assert!(!s.contains("{{"), "{file}: unrendered placeholder in `{s}`");
            }
        }
    }
}
