#[cfg(target_os = "windows")]
pub const FRPC_PATH: &str = "VieShare/bin/frpc.exe";

#[cfg(target_os = "linux")]
pub const FRPC_PATH: &str = "VieShare/bin/frpc";
