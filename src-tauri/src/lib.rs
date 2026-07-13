mod engine;

use engine::plan::TestPlan;
use engine::runner;
use tauri::AppHandle;

#[tauri::command]
async fn start_test(app_handle: AppHandle, plan_json: String) -> Result<(), String> {
    let plan: TestPlan = serde_json::from_str(&plan_json)
        .map_err(|e| format!("Failed to parse test plan: {}", e))?;
    runner::start_test_plan(plan, app_handle).await
}

#[tauri::command]
fn stop_test() {
    runner::stop_test_plan();
}

#[tauri::command]
fn get_response_body(id: String) -> String {
    runner::get_response_body(&id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![start_test, stop_test, get_response_body])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
