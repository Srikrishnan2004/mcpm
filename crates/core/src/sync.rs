//! Applies project bindings to client config files.
//!
//! Per file: compute desired entries, compare with what's on disk and what mcpm owns
//! (manifest), back up, write, update manifest. User-owned entries are never modified.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::adapters::{self, ClientAdapter};
use crate::model::{ClientId, Project};
use crate::store::{DataDir, EntryRecord, FileRecord, Manifest, MANIFEST};
use crate::{Error, Result};

/// `command` written into project-scoped configs. Bare name so committed files stay portable.
pub const PROJECT_SHIM: &str = "mcpm";

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncReport {
    pub writes: Vec<FileWrite>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWrite {
    pub client: ClientId,
    pub path: String,
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub removed: Vec<String>,
    pub reload_hint: &'static str,
    pub backup: Option<String>,
}

pub fn sync_project(data: &DataDir, project: &Project) -> Result<SyncReport> {
    let root = PathBuf::from(&project.path);

    // file -> (client, entry -> instance)
    let mut desired: BTreeMap<PathBuf, (ClientId, BTreeMap<String, String>)> = BTreeMap::new();
    for binding in &project.bindings {
        for &client in &binding.clients {
            let adapter = adapters::get(client)?;
            let Some(rel) = adapter.project_file() else {
                continue;
            };
            desired
                .entry(root.join(rel))
                .or_insert_with(|| (client, BTreeMap::new()))
                .1
                .insert(binding.entry.clone(), binding.instance.clone());
        }
    }

    let mut manifest: Manifest = data.load(MANIFEST)?;

    // Files we wrote before under this project but which no binding targets anymore.
    for (path, record) in &manifest.files {
        let path = PathBuf::from(path);
        if path.starts_with(&root) && !desired.contains_key(&path) {
            desired.insert(path, (record.client, BTreeMap::new()));
        }
    }

    let mut report = SyncReport::default();
    let mut result = Ok(());
    for (path, (client, entries)) in &desired {
        let adapter = adapters::get(*client)?;
        match apply_file(
            data,
            &mut manifest,
            adapter.as_ref(),
            path,
            entries,
            PROJECT_SHIM,
        ) {
            Ok(Some(write)) => report.writes.push(write),
            Ok(None) => {}
            Err(e) => {
                result = Err(e);
                break;
            }
        }
    }
    // Persist ownership even on partial failure so already-written entries stay tracked.
    data.save(MANIFEST, &manifest)?;
    result.map(|()| report)
}

fn apply_file(
    data: &DataDir,
    manifest: &mut Manifest,
    adapter: &dyn ClientAdapter,
    path: &Path,
    desired: &BTreeMap<String, String>,
    shim: &str,
) -> Result<Option<FileWrite>> {
    let key = path.display().to_string();
    let current = adapter.read_entries(path)?;
    let owned = manifest
        .files
        .get(&key)
        .map(|r| r.entries.clone())
        .unwrap_or_default();

    let mut upserts = Vec::new();
    let (mut added, mut updated, mut removed) = (Vec::new(), Vec::new(), Vec::new());
    let mut records = BTreeMap::new();

    for (entry, instance) in desired {
        let want = adapter.shim_entry(shim, instance);
        match current.get(entry) {
            None => {
                added.push(entry.clone());
                upserts.push((entry.clone(), want.clone()));
            }
            Some(have) if *have == want => {}
            // TODO(v0.2): surface drift (hand-edited owned entry) instead of overwriting.
            Some(_) if owned.contains_key(entry) => {
                updated.push(entry.clone());
                upserts.push((entry.clone(), want.clone()));
            }
            Some(_) => {
                return Err(Error::Collision {
                    entry: entry.clone(),
                    path: path.to_path_buf(),
                })
            }
        }
        records.insert(
            entry.clone(),
            EntryRecord {
                instance: instance.clone(),
                value: want,
            },
        );
    }

    for entry in owned.keys() {
        if !desired.contains_key(entry) && current.contains_key(entry) {
            removed.push(entry.clone());
        }
    }

    let changed = !(upserts.is_empty() && removed.is_empty());
    let backup = if changed && path.exists() {
        Some(backup_file(data, adapter.id(), path)?)
    } else {
        None
    };
    if changed {
        adapter.write_entries(path, &upserts, &removed)?;
    }

    if records.is_empty() {
        manifest.files.remove(&key);
    } else {
        manifest.files.insert(
            key.clone(),
            FileRecord {
                client: adapter.id(),
                entries: records,
            },
        );
    }

    Ok(changed.then(|| FileWrite {
        client: adapter.id(),
        path: key,
        added,
        updated,
        removed,
        reload_hint: adapter.reload_hint(),
        backup: backup.map(|p| p.display().to_string()),
    }))
}

fn backup_file(data: &DataDir, client: ClientId, path: &Path) -> Result<PathBuf> {
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
    let name: String = path
        .display()
        .to_string()
        .chars()
        .map(|c| {
            if matches!(c, ':' | '\\' | '/') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let dir = data
        .path("backups")
        .join(stamp)
        .join(format!("{client:?}").to_lowercase());
    fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
    let target = dir.join(name);
    fs::copy(path, &target).map_err(|e| Error::io(path, e))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Binding;

    fn project(root: &Path, bindings: Vec<Binding>) -> Project {
        Project {
            id: "p1".into(),
            path: root.display().to_string(),
            name: "demo".into(),
            bindings,
        }
    }

    fn binding(instance: &str) -> Binding {
        Binding {
            entry: "supabase".into(),
            instance: instance.into(),
            clients: vec![ClientId::Cursor, ClientId::ClaudeCode],
        }
    }

    #[test]
    fn bind_remap_unbind() {
        let tmp = tempfile::tempdir().unwrap();
        let data = DataDir::at(tmp.path().join("data"));
        let root = tmp.path().join("proj");
        fs::create_dir_all(&root).unwrap();

        let r = sync_project(&data, &project(&root, vec![binding("supabase@a")])).unwrap();
        assert_eq!(r.writes.len(), 2);
        let cursor = root.join(".cursor/mcp.json");
        let text = fs::read_to_string(&cursor).unwrap();
        assert!(text.contains("supabase@a"));

        // Remap
        let r = sync_project(&data, &project(&root, vec![binding("supabase@b")])).unwrap();
        assert!(r
            .writes
            .iter()
            .all(|w| w.updated == ["supabase"] && w.backup.is_some()));
        assert!(fs::read_to_string(&cursor).unwrap().contains("supabase@b"));

        // Idempotent
        let r = sync_project(&data, &project(&root, vec![binding("supabase@b")])).unwrap();
        assert!(r.writes.is_empty());

        // Unbind removes only our entry
        let r = sync_project(&data, &project(&root, vec![])).unwrap();
        assert!(r.writes.iter().all(|w| w.removed == ["supabase"]));
        assert!(!fs::read_to_string(&cursor).unwrap().contains("supabase"));
    }

    #[test]
    fn refuses_to_overwrite_user_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let data = DataDir::at(tmp.path().join("data"));
        let root = tmp.path().join("proj");
        fs::create_dir_all(root.join(".cursor")).unwrap();
        fs::write(
            root.join(".cursor/mcp.json"),
            r#"{"mcpServers":{"supabase":{"command":"npx"}}}"#,
        )
        .unwrap();

        let err = sync_project(&data, &project(&root, vec![binding("supabase@a")])).unwrap_err();
        assert!(matches!(err, Error::Collision { .. }));
    }
}
