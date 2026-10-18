//! Turns a template + instance into a concrete launch spec.
//!
//! Placeholder syntax inside template strings:
//! - `{{setting}}`          value of a setting
//! - `{{secret:key}}`       secret from the keychain (only allowed in `env` and `headers`)
//! - `{{#flag}}…{{/flag}}`  included only when the setting is truthy

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::model::{Instance, Template, TransportKind};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "transport", rename_all = "lowercase")]
pub enum LaunchSpec {
    Stdio {
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
    },
    Http {
        url: String,
        headers: BTreeMap<String, String>,
    },
}

/// Settings from the instance, falling back to template defaults. Errors on missing required ones.
pub fn effective_settings(
    template: &Template,
    instance_settings: &BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Value>> {
    let mut out = BTreeMap::new();
    for spec in &template.settings {
        let value = instance_settings
            .get(&spec.key)
            .filter(|v| !is_blank(v))
            .cloned()
            .or_else(|| spec.default.clone());
        match value {
            Some(v) => {
                out.insert(spec.key.clone(), v);
            }
            None if spec.required => {
                return Err(Error::Invalid(format!(
                    "missing required setting `{}`",
                    spec.key
                )))
            }
            None => {}
        }
    }
    Ok(out)
}

pub fn launch_spec(
    template: &Template,
    instance: &Instance,
    secret: &dyn Fn(&str) -> Result<String>,
) -> Result<LaunchSpec> {
    let settings = effective_settings(template, &instance.settings)?;
    let ctx = Ctx {
        settings: &settings,
        secret,
    };
    let missing = |kind: &str| {
        Error::Invalid(format!(
            "template `{}` has no {kind} transport",
            template.id
        ))
    };

    match instance.transport {
        TransportKind::Stdio => {
            let spec = template
                .transports
                .stdio
                .as_ref()
                .ok_or_else(|| missing("stdio"))?;
            let args = spec
                .args
                .iter()
                .map(|a| render(a, &ctx, false))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter(|a| !a.is_empty())
                .collect();
            Ok(LaunchSpec::Stdio {
                command: render(&spec.command, &ctx, false)?,
                args,
                env: render_map(&spec.env, &ctx)?,
            })
        }
        TransportKind::Http => {
            let spec = template
                .transports
                .http
                .as_ref()
                .ok_or_else(|| missing("http"))?;
            Ok(LaunchSpec::Http {
                url: render(&spec.url, &ctx, false)?,
                headers: render_map(&spec.headers, &ctx)?,
            })
        }
    }
}

struct Ctx<'a> {
    settings: &'a BTreeMap<String, Value>,
    secret: &'a dyn Fn(&str) -> Result<String>,
}

fn render_map(map: &BTreeMap<String, String>, ctx: &Ctx) -> Result<BTreeMap<String, String>> {
    map.iter()
        .map(|(k, v)| Ok((k.clone(), render(v, ctx, true)?)))
        .collect()
}

fn render(input: &str, ctx: &Ctx, allow_secrets: bool) -> Result<String> {
    let unclosed = || Error::Invalid(format!("unclosed placeholder in `{input}`"));
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find("}}").ok_or_else(unclosed)?;
        let tag = after[..end].trim();
        rest = &after[end + 2..];

        if let Some(name) = tag.strip_prefix('#') {
            let close = format!("{{{{/{name}}}}}");
            let close_at = rest.find(&close).ok_or_else(unclosed)?;
            if truthy(ctx.settings.get(name)) {
                out.push_str(&render(&rest[..close_at], ctx, allow_secrets)?);
            }
            rest = &rest[close_at + close.len()..];
        } else if let Some(key) = tag.strip_prefix("secret:") {
            if !allow_secrets {
                return Err(Error::Invalid(format!(
                    "secret `{key}` may only be used in env or headers"
                )));
            }
            out.push_str(&(ctx.secret)(key)?);
        } else {
            out.push_str(&ctx.settings.get(tag).map(as_text).unwrap_or_default());
        }
    }
    out.push_str(rest);
    Ok(out)
}

fn is_blank(v: &Value) -> bool {
    matches!(v, Value::Null) || matches!(v, Value::String(s) if s.trim().is_empty())
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Number(_)) | Some(Value::Object(_)) => true,
    }
}

fn as_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use serde_json::json;

    fn supabase() -> Template {
        serde_json::from_str(include_str!("../../../catalog/servers/supabase.json")).unwrap()
    }

    fn instance(settings: Value) -> Instance {
        Instance {
            id: "supabase@a".into(),
            template: "supabase".into(),
            label: String::new(),
            transport: TransportKind::Stdio,
            settings: serde_json::from_value(settings).unwrap(),
            secrets: vec!["access_token".into()],
            created_at: String::new(),
        }
    }

    #[test]
    fn renders_stdio_with_flags_and_secrets() {
        let spec = launch_spec(
            &supabase(),
            &instance(json!({ "project_ref": "abc", "read_only": true })),
            &|key| Ok(format!("<{key}>")),
        )
        .unwrap();
        let LaunchSpec::Stdio { args, env, .. } = spec else {
            panic!("expected stdio")
        };
        assert!(args.contains(&"--project-ref=abc".to_string()));
        assert!(args.contains(&"--read-only".to_string()));
        assert_eq!(env["SUPABASE_ACCESS_TOKEN"], "<access_token>");
    }

    #[test]
    fn drops_empty_flag_args() {
        let spec = launch_spec(
            &supabase(),
            &instance(json!({ "project_ref": "abc", "read_only": false })),
            &|_| Ok("t".into()),
        )
        .unwrap();
        let LaunchSpec::Stdio { args, .. } = spec else {
            panic!("expected stdio")
        };
        assert!(!args.iter().any(|a| a.is_empty() || a == "--read-only"));
    }

    #[test]
    fn missing_required_setting_errors() {
        let err = launch_spec(&supabase(), &instance(json!({})), &|_| Ok("t".into()));
        assert!(err.is_err());
    }

    #[test]
    fn secrets_not_allowed_in_args() {
        let mut t = supabase();
        t.transports.stdio.as_mut().unwrap().args = vec!["{{secret:access_token}}".into()];
        let err = launch_spec(&t, &instance(json!({ "project_ref": "x" })), &|_| {
            Ok("t".into())
        });
        assert!(err.is_err());
    }
}
