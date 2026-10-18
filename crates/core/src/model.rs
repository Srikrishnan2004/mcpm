use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Error, Result};

/// A server *type* from the catalog (or user-defined), e.g. `supabase`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub transports: Transports,
    pub default_transport: TransportKind,
    #[serde(default)]
    pub settings: Vec<SettingSpec>,
    #[serde(default)]
    pub secrets: Vec<SecretSpec>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Transports {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdio: Option<StdioSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioSpec {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSpec {
    pub url: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    Stdio,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingSpec {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: SettingKind,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingKind {
    String,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSpec {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// A named, configured copy of a template: `supabase@clientA`.
/// Secret *values* are never stored here, only the keys that exist in the keychain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub template: String,
    #[serde(default)]
    pub label: String,
    pub transport: TransportKind,
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
    #[serde(default)]
    pub secrets: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientId {
    ClaudeCode,
    ClaudeDesktop,
    Cursor,
    Vscode,
    Windsurf,
    Antigravity,
    Codex,
    GeminiCli,
}

/// A folder registered in mcpm.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub path: String,
    pub name: String,
    #[serde(default)]
    pub bindings: Vec<Binding>,
}

/// Entry name (what the IDE sees) -> instance, written to the selected clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub entry: String,
    pub instance: String,
    pub clients: Vec<ClientId>,
}

/// Lowercase letters, digits and dashes; must not start with a dash.
/// Underscores are rejected because some clients build tool names as `mcp_<server>_<tool>`.
pub fn is_valid_slug(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn instance_id(template: &str, name: &str) -> Result<String> {
    if !is_valid_slug(template) || !is_valid_slug(name) {
        return Err(Error::Invalid(format!(
            "invalid instance name `{template}@{name}`: use lowercase letters, digits and dashes"
        )));
    }
    Ok(format!("{template}@{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs() {
        assert!(is_valid_slug("client-a"));
        assert!(is_valid_slug("prod2"));
        assert!(!is_valid_slug("Client"));
        assert!(!is_valid_slug("client_a"));
        assert!(!is_valid_slug("-a"));
        assert!(!is_valid_slug(""));
        assert_eq!(
            instance_id("supabase", "client-a").unwrap(),
            "supabase@client-a"
        );
    }
}
