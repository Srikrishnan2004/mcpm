//! Tauri shell: exposes mcpm-core operations as commands. No business logic here.

use std::sync::Mutex;

use mcpm_core::adapters::ClientInfo;
use mcpm_core::model::{ClientId, Instance, Project, Template};
use mcpm_core::secrets::KeyringStore;
use mcpm_core::service::{Mcpm, NewInstance};
use mcpm_core::store::DataDir;
use mcpm_core::sync::SyncReport;
use tauri::State;

struct AppState(Mutex<Mcpm>);

type CmdResult<T> = Result<T, String>;

fn with<T>(state: &State<AppState>, f: impl FnOnce(&Mcpm) -> mcpm_core::Result<T>) -> CmdResult<T> {
    let mcpm = state.0.lock().map_err(|e| e.to_string())?;
    f(&mcpm).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_templates(state: State<AppState>) -> CmdResult<Vec<Template>> {
    with(&state, |m| Ok(m.templates()))
}

#[tauri::command]
fn list_clients(state: State<AppState>) -> CmdResult<Vec<ClientInfo>> {
    with(&state, |m| Ok(m.clients()))
}

#[tauri::command]
fn list_instances(state: State<AppState>) -> CmdResult<Vec<Instance>> {
    with(&state, |m| m.instances())
}

#[tauri::command]
fn create_instance(state: State<AppState>, input: NewInstance) -> CmdResult<Instance> {
    with(&state, |m| m.create_instance(input))
}

#[tauri::command]
fn delete_instance(state: State<AppState>, id: String) -> CmdResult<()> {
    with(&state, |m| m.delete_instance(&id))
}

#[tauri::command]
fn set_secret(state: State<AppState>, id: String, key: String, value: String) -> CmdResult<()> {
    with(&state, |m| m.set_secret(&id, &key, &value))
}

#[tauri::command]
fn list_projects(state: State<AppState>) -> CmdResult<Vec<Project>> {
    with(&state, |m| m.projects())
}

#[tauri::command]
fn add_project(state: State<AppState>, path: String) -> CmdResult<Project> {
    with(&state, |m| m.add_project(&path))
}

#[tauri::command]
fn remove_project(state: State<AppState>, id: String) -> CmdResult<SyncReport> {
    with(&state, |m| m.remove_project(&id))
}

#[tauri::command]
fn set_binding(
    state: State<AppState>,
    project_id: String,
    entry: String,
    instance: String,
    clients: Vec<ClientId>,
) -> CmdResult<SyncReport> {
    with(&state, |m| {
        m.set_binding(&project_id, &entry, &instance, clients)
    })
}

#[tauri::command]
fn remove_binding(
    state: State<AppState>,
    project_id: String,
    entry: String,
) -> CmdResult<SyncReport> {
    with(&state, |m| m.remove_binding(&project_id, &entry))
}

#[tauri::command]
fn sync_project(state: State<AppState>, project_id: String) -> CmdResult<SyncReport> {
    with(&state, |m| m.sync(&project_id))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data = DataDir::from_env().expect("cannot resolve the mcpm data directory");
    let mcpm = Mcpm::open(data, Box::new(KeyringStore)).expect("cannot open mcpm data");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Mutex::new(mcpm)))
        .invoke_handler(tauri::generate_handler![
            list_templates,
            list_clients,
            list_instances,
            create_instance,
            delete_instance,
            set_secret,
            list_projects,
            add_project,
            remove_project,
            set_binding,
            remove_binding,
            sync_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running mcpm");
}
