// Threadbear's desktop app: a Tauri shell around a web interface. The Rust
// side owns the runner connection (../client, shared with the GPUI app);
// the interface asks it to make calls and listens for what the runner pushes.
use threadbear_client::config;
use threadbear_client::runner::{Client, RpcError, Sink, Status};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

struct TauriSink(AppHandle);

impl Sink for TauriSink {
    fn status(&self, status: &Status) {
        let _ = self.0.emit("runner-status", status);
    }
    fn notification(&self, method: &str, params: &Value) {
        let _ = self.0.emit("runner-notification", json!({ "method": method, "params": params }));
    }
}

struct Runner(Arc<Client>);

#[tauri::command]
fn runner_status(runner: State<'_, Runner>) -> Status {
    runner.0.status()
}

#[tauri::command]
async fn runner_call(runner: State<'_, Runner>, method: String, params: Option<Value>) -> Result<Value, RpcError> {
    let client = runner.0.clone();
    client.call(&method, params.unwrap_or_else(|| json!({}))).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let client = Client::new(config::RunnerConfig::from_env(), TauriSink(app.handle().clone()));
            tauri::async_runtime::spawn(client.clone().run());
            app.manage(Runner(client));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![runner_status, runner_call])
        .run(tauri::generate_context!())
        .expect("error while running Threadbear");
}
