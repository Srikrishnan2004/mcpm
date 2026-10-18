//! `mcpm` CLI.
//!
//! `mcpm run <instance>` is what AI clients spawn. stdout belongs to the MCP protocol,
//! so this binary must only ever write diagnostics to stderr.

use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};
use mcpm_core::render::LaunchSpec;
use mcpm_core::secrets::KeyringStore;
use mcpm_core::service::Mcpm;
use mcpm_core::store::DataDir;

#[derive(Parser)]
#[command(name = "mcpm", version, about = "Manage MCP servers across AI clients")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Launch an instance (this is what client configs call)
    Run { instance: String },
    /// List configured instances
    List,
    /// List supported clients and where their configs live
    Clients,
    /// Store a secret for an instance in the OS keychain (value read from stdin)
    SecretSet { instance: String, key: String },
    /// Re-apply bindings for a registered project folder
    Sync { path: PathBuf },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("mcpm: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mcpm = Mcpm::open(DataDir::from_env()?, Box::new(KeyringStore))?;

    match cli.command {
        Cmd::Run { instance } => return launch(&mcpm, &instance),
        Cmd::List => {
            for i in mcpm.instances()? {
                println!(
                    "{:<32} {:<6} {}",
                    i.id,
                    format!("{:?}", i.transport).to_lowercase(),
                    i.label
                );
            }
        }
        Cmd::Clients => {
            for c in mcpm.clients() {
                println!(
                    "{:<26} {:<9} {}",
                    c.name,
                    if c.detected { "detected" } else { "-" },
                    c.global_path.unwrap_or_default()
                );
            }
        }
        Cmd::SecretSet { instance, key } => {
            let mut value = String::new();
            io::stdin().lock().read_line(&mut value)?;
            mcpm.set_secret(&instance, &key, value.trim_end_matches(['\r', '\n']))?;
            eprintln!("stored `{key}` for `{instance}` in the OS keychain");
        }
        Cmd::Sync { path } => {
            let project = mcpm.find_project_by_path(&path)?;
            let report = mcpm.sync(&project.id)?;
            if report.writes.is_empty() {
                eprintln!("already in sync");
            }
            for w in report.writes {
                eprintln!(
                    "{} ({:?}): +{:?} ~{:?} -{:?}",
                    w.path, w.client, w.added, w.updated, w.removed
                );
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Spawn the real server with secrets injected; stdio is inherited so bytes pass straight through.
fn launch(mcpm: &Mcpm, instance: &str) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match mcpm.launch_spec(instance)? {
        LaunchSpec::Stdio { command, args, env } => {
            // Resolves `npx` -> `npx.cmd` on Windows via PATHEXT.
            let program = which::which(&command).unwrap_or_else(|_| PathBuf::from(&command));
            // TODO(v0.3): Windows Job Object so the child dies with the shim.
            let status = Command::new(program)
                .args(&args)
                .envs(&env)
                .status()
                .map_err(|e| format!("failed to start `{command}`: {e}"))?;
            Ok(ExitCode::from(
                status.code().unwrap_or(1).clamp(0, 255) as u8
            ))
        }
        LaunchSpec::Http { url, .. } => Err(format!(
            "remote transport ({url}) is not supported by the shim yet (planned for v0.3)"
        )
        .into()),
    }
}
