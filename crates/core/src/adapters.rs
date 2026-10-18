//! Client adapters: where each AI client keeps its MCP config and how entries are shaped.
//!
//! Add a client by implementing [`ClientAdapter`] and registering it in [`all`].
//! Most clients use a JSON file with a map of servers, which [`JsonClient`] covers.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::model::ClientId;
use crate::store::write_json_atomic;
use crate::{Error, Result};

pub trait ClientAdapter: Send + Sync {
    fn id(&self) -> ClientId;
    fn name(&self) -> &'static str;
    /// Candidate user-level config files, most current first.
    fn global_candidates(&self) -> Vec<PathBuf>;
    /// Directories whose presence means the client is installed.
    fn markers(&self) -> Vec<PathBuf>;
    /// Project-level config file, relative to the project root.
    fn project_file(&self) -> Option<&'static str>;
    fn reload_hint(&self) -> &'static str;
    /// The entry mcpm writes: launches the instance through the shim.
    fn shim_entry(&self, command: &str, instance: &str) -> Value;
    fn read_entries(&self, path: &Path) -> Result<BTreeMap<String, Value>>;
    fn write_entries(
        &self,
        path: &Path,
        upserts: &[(String, Value)],
        removes: &[String],
    ) -> Result<()>;

    fn global_file(&self) -> Option<PathBuf> {
        let candidates = self.global_candidates();
        candidates
            .iter()
            .find(|p| p.exists())
            .cloned()
            .or_else(|| candidates.into_iter().next())
    }

    fn detected(&self) -> bool {
        self.global_candidates().iter().any(|p| p.exists())
            || self.markers().iter().any(|p| p.exists())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub id: ClientId,
    pub name: &'static str,
    pub detected: bool,
    pub global_path: Option<String>,
    pub project_file: Option<&'static str>,
    pub reload_hint: &'static str,
}

pub fn info(adapter: &dyn ClientAdapter) -> ClientInfo {
    ClientInfo {
        id: adapter.id(),
        name: adapter.name(),
        detected: adapter.detected(),
        global_path: adapter.global_file().map(|p| p.display().to_string()),
        project_file: adapter.project_file(),
        reload_hint: adapter.reload_hint(),
    }
}

/// A client whose config is a JSON object with a map of servers under `key`.
pub struct JsonClient {
    pub id: ClientId,
    pub name: &'static str,
    /// `mcpServers` for most clients, `servers` for VS Code.
    pub key: &'static str,
    /// Whether stdio entries carry `"type": "stdio"`.
    pub typed: bool,
    pub global: fn() -> Vec<PathBuf>,
    pub markers: fn() -> Vec<PathBuf>,
    pub project: Option<&'static str>,
    pub reload: &'static str,
}

impl JsonClient {
    fn read_doc(&self, path: &Path) -> Result<Map<String, Value>> {
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Map::new()),
            Err(e) => return Err(Error::io(path, e)),
        };
        if text.trim().is_empty() {
            return Ok(Map::new());
        }
        // TODO(v0.2): format-preserving JSONC editing so comments survive.
        match serde_json::from_str(&text).map_err(|e| Error::json(path, e))? {
            Value::Object(map) => Ok(map),
            _ => Err(Error::Invalid(format!(
                "{} is not a JSON object",
                path.display()
            ))),
        }
    }
}

impl ClientAdapter for JsonClient {
    fn id(&self) -> ClientId {
        self.id
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn global_candidates(&self) -> Vec<PathBuf> {
        (self.global)()
    }

    fn markers(&self) -> Vec<PathBuf> {
        (self.markers)()
    }

    fn project_file(&self) -> Option<&'static str> {
        self.project
    }

    fn reload_hint(&self) -> &'static str {
        self.reload
    }

    fn shim_entry(&self, command: &str, instance: &str) -> Value {
        if self.typed {
            json!({ "type": "stdio", "command": command, "args": ["run", instance] })
        } else {
            json!({ "command": command, "args": ["run", instance] })
        }
    }

    fn read_entries(&self, path: &Path) -> Result<BTreeMap<String, Value>> {
        let doc = self.read_doc(path)?;
        Ok(match doc.get(self.key) {
            Some(Value::Object(servers)) => servers
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            _ => BTreeMap::new(),
        })
    }

    fn write_entries(
        &self,
        path: &Path,
        upserts: &[(String, Value)],
        removes: &[String],
    ) -> Result<()> {
        let mut doc = self.read_doc(path)?;
        let servers = doc
            .entry(self.key)
            .or_insert_with(|| Value::Object(Map::new()));
        let Value::Object(servers) = servers else {
            return Err(Error::Invalid(format!(
                "`{}` in {} is not an object",
                self.key,
                path.display()
            )));
        };
        for name in removes {
            servers.shift_remove(name);
        }
        for (name, value) in upserts {
            servers.insert(name.clone(), value.clone());
        }
        write_json_atomic(path, &Value::Object(doc))
    }
}

fn home(rel: &str) -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(rel)
}

/// `%APPDATA%` on Windows, `~/Library/Application Support` on macOS, `~/.config` on Linux.
fn config(rel: &str) -> PathBuf {
    dirs::config_dir().unwrap_or_default().join(rel)
}

pub fn all() -> Vec<Box<dyn ClientAdapter>> {
    vec![
        Box::new(JsonClient {
            id: ClientId::ClaudeCode,
            name: "Claude Code",
            key: "mcpServers",
            typed: true,
            global: || vec![home(".claude.json")],
            markers: || vec![home(".claude")],
            project: Some(".mcp.json"),
            reload: "Start a new session or run /mcp to reconnect",
        }),
        Box::new(JsonClient {
            id: ClientId::ClaudeDesktop,
            name: "Claude Desktop",
            key: "mcpServers",
            typed: false,
            global: || vec![config("Claude/claude_desktop_config.json")],
            markers: || vec![config("Claude")],
            project: None,
            reload: "Quit and reopen Claude Desktop",
        }),
        Box::new(JsonClient {
            id: ClientId::Cursor,
            name: "Cursor",
            key: "mcpServers",
            typed: true,
            global: || vec![home(".cursor/mcp.json")],
            markers: || vec![home(".cursor")],
            project: Some(".cursor/mcp.json"),
            reload: "Restart Cursor",
        }),
        Box::new(JsonClient {
            id: ClientId::Vscode,
            name: "VS Code",
            key: "servers",
            typed: true,
            global: || vec![config("Code/User/mcp.json")],
            markers: || vec![config("Code")],
            project: Some(".vscode/mcp.json"),
            reload: "Reloads automatically",
        }),
        Box::new(JsonClient {
            id: ClientId::Windsurf,
            name: "Windsurf / Devin Desktop",
            key: "mcpServers",
            typed: false,
            global: || {
                vec![
                    config("devin/mcp_config.json"),
                    home(".codeium/mcp_config.json"),
                    home(".codeium/windsurf/mcp_config.json"),
                ]
            },
            markers: || vec![config("devin"), home(".codeium")],
            project: None,
            reload: "Refresh MCP servers in the Cascade panel",
        }),
        Box::new(JsonClient {
            id: ClientId::Antigravity,
            name: "Antigravity",
            key: "mcpServers",
            typed: false,
            global: || {
                vec![
                    home(".gemini/config/mcp_config.json"),
                    home(".gemini/antigravity/mcp_config.json"),
                ]
            },
            markers: || vec![home(".gemini/config"), home(".gemini/antigravity")],
            project: Some(".agents/mcp_config.json"),
            reload: "Use the refresh button or /mcp",
        }),
        Box::new(JsonClient {
            id: ClientId::GeminiCli,
            name: "Gemini CLI",
            key: "mcpServers",
            typed: false,
            global: || vec![home(".gemini/settings.json")],
            markers: || vec![home(".gemini")],
            project: Some(".gemini/settings.json"),
            reload: "Run /mcp refresh",
        }),
        // TODO(v0.2): Codex adapter (TOML, `[mcp_servers.<name>]`, via toml_edit).
    ]
}

pub fn get(id: ClientId) -> Result<Box<dyn ClientAdapter>> {
    all()
        .into_iter()
        .find(|a| a.id() == id)
        .ok_or_else(|| Error::NotFound(format!("client adapter `{id:?}`")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_keeps_user_entries_and_other_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(
            &path,
            r#"{"theme":"dark","mcpServers":{"mine":{"command":"node","args":["x.js"]}}}"#,
        )
        .unwrap();

        let cursor = get(ClientId::Cursor).unwrap();
        let entry = cursor.shim_entry("mcpm", "supabase@a");
        cursor
            .write_entries(&path, &[("supabase".into(), entry.clone())], &[])
            .unwrap();

        let doc: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(doc["theme"], "dark");
        assert_eq!(doc["mcpServers"]["mine"]["command"], "node");
        assert_eq!(doc["mcpServers"]["supabase"], entry);

        cursor
            .write_entries(&path, &[], &["supabase".into()])
            .unwrap();
        let entries = cursor.read_entries(&path).unwrap();
        assert!(entries.contains_key("mine") && !entries.contains_key("supabase"));
    }

    #[test]
    fn vscode_uses_servers_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        let vscode = get(ClientId::Vscode).unwrap();
        vscode
            .write_entries(
                &path,
                &[("gh".into(), vscode.shim_entry("mcpm", "github@me"))],
                &[],
            )
            .unwrap();
        let doc: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(doc["servers"]["gh"]["type"], "stdio");
    }
}
