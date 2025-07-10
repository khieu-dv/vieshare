use futures_util::join;
use tauri::AppHandle;

use crate::{
    bin::FRPC_PATH, bootstrap::frpc::bootstrap_frpc, logger::logger::IPCLogger,
    utils::file::file_exists,
};

#[tauri::command(async)]
pub async fn bootstrap_install(app: AppHandle) {
    let frpc_logger = IPCLogger::new(app);

    // Chạy song song cả 3 tiến trình bootstrap
    join!(bootstrap_frpc(&frpc_logger),);
}

#[tauri::command]
pub fn bootstrap_check() -> bool {
    file_exists(FRPC_PATH)
}
