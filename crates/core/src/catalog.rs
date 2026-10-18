//! Server templates: built-ins from `catalog/servers/*.json` plus user templates
//! from `<data>/templates/*.json` (user templates override built-ins with the same id).

use std::collections::BTreeMap;
use std::fs;

use crate::model::Template;
use crate::store::DataDir;
use crate::{Error, Result};

const BUILTIN: &[(&str, &str)] = &[
    (
        "filesystem.json",
        include_str!("../../../catalog/servers/filesystem.json"),
    ),
    (
        "github.json",
        include_str!("../../../catalog/servers/github.json"),
    ),
    (
        "supabase.json",
        include_str!("../../../catalog/servers/supabase.json"),
    ),
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
    #[test]
    fn builtins_parse() {
        let c = super::Catalog::builtin().unwrap();
        assert!(c.get("supabase").is_ok());
    }
}
