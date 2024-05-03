use sqlx::SqlitePool;

mod db;

pub struct AppState {
    pub pool: SqlitePool,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            pool: db::establish_connection().await,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let state = AppState::new().await;

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![db::message::get_messages])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
