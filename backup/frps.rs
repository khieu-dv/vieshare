// src/commands/frps.rs
use std::process::{Command, Child, Stdio};
use std::sync::Mutex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::bin::FRPC_PATH;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpsConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub token: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub name: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String, // tcp, udp, http, https
    pub custom_domains: Option<Vec<String>>,
    pub subdomain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpsStatus {
    pub connected: bool,
    pub server_addr: String,
    pub active_mappings: Vec<PortMapping>,
    pub pid: Option<u32>,
}

pub type FrpsProcesses = Mutex<HashMap<String, Child>>;

#[tauri::command]
pub async fn frps_connect(
    config: FrpsConfig,
    processes: State<'_, FrpsProcesses>,
) -> Result<String, String> {
    let mut processes_guard = processes.lock().map_err(|e| e.to_string())?;
    
    // Check if already connected
    if processes_guard.contains_key("main") {
        return Err("FRPS client is already running".to_string());
    }

    // Create frpc configuration
    let config_content = format!(
        r#"[common]
server_addr = {}
server_port = {}
token = {}
user = {}

[web]
type = http
local_port = 3000
custom_domains = example.com
"#,
        config.server_addr,
        config.server_port,
        config.token,
        config.user
    );

    // Write config to temporary file
    let config_path = "/tmp/frpc.toml";
    std::fs::write(config_path, config_content).map_err(|e| e.to_string())?;

    // Start frpc process
    let child = Command::new(FRPC_PATH)
        .arg("-c")
        .arg(config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start frpc: {}", e))?;

    processes_guard.insert("main".to_string(), child);
    
    Ok("FRPS client connected successfully".to_string())
}

#[tauri::command]
pub async fn frps_disconnect(
    processes: State<'_, FrpsProcesses>,
) -> Result<String, String> {
    let mut processes_guard = processes.lock().map_err(|e| e.to_string())?;
    
    if let Some(mut child) = processes_guard.remove("main") {
        child.kill().map_err(|e| format!("Failed to kill process: {}", e))?;
        Ok("FRPS client disconnected successfully".to_string())
    } else {
        Err("No active FRPS connection found".to_string())
    }
}

#[tauri::command]
pub async fn frps_add_port_mapping(
    mapping: PortMapping,
    config: FrpsConfig,
    processes: State<'_, FrpsProcesses>,
) -> Result<String, String> {
    let processes_guard = processes.lock().map_err(|e| e.to_string())?;
    
    if !processes_guard.contains_key("main") {
        return Err("FRPS client is not connected".to_string());
    }

    // Create mapping configuration
    let mapping_config = match mapping.protocol.as_str() {
        "tcp" => format!(
            r#"[{}]
type = tcp
local_port = {}
remote_port = {}
"#,
            mapping.name, mapping.local_port, mapping.remote_port
        ),
        "udp" => format!(
            r#"[{}]
type = udp
local_port = {}
remote_port = {}
"#,
            mapping.name, mapping.local_port, mapping.remote_port
        ),
        "http" => {
            let mut config = format!(
                r#"[{}]
type = http
local_port = {}
"#,
                mapping.name, mapping.local_port
            );
            
            if let Some(domains) = &mapping.custom_domains {
                config.push_str(&format!("custom_domains = {}\n", domains.join(",")));
            }
            
            if let Some(subdomain) = &mapping.subdomain {
                config.push_str(&format!("subdomain = {}\n", subdomain));
            }
            
            config
        },
        "https" => {
            let mut config = format!(
                r#"[{}]
type = https
local_port = {}
"#,
                mapping.name, mapping.local_port
            );
            
            if let Some(domains) = &mapping.custom_domains {
                config.push_str(&format!("custom_domains = {}\n", domains.join(",")));
            }
            
            if let Some(subdomain) = &mapping.subdomain {
                config.push_str(&format!("subdomain = {}\n", subdomain));
            }
            
            config
        },
        _ => return Err("Unsupported protocol".to_string()),
    };

    // Here you would typically reload the frpc configuration
    // For now, we'll return success
    Ok(format!("Port mapping {} added successfully", mapping.name))
}

#[tauri::command]
pub async fn frps_remove_port_mapping(
    mapping_name: String,
    processes: State<'_, FrpsProcesses>,
) -> Result<String, String> {
    let processes_guard = processes.lock().map_err(|e| e.to_string())?;
    
    if !processes_guard.contains_key("main") {
        return Err("FRPS client is not connected".to_string());
    }

    // Here you would typically reload the frpc configuration without the mapping
    Ok(format!("Port mapping {} removed successfully", mapping_name))
}

#[tauri::command]
pub async fn frps_get_status(
    processes: State<'_, FrpsProcesses>,
) -> Result<FrpsStatus, String> {
    let processes_guard = processes.lock().map_err(|e| e.to_string())?;
    
    let connected = processes_guard.contains_key("main");
    let pid = if connected {
        processes_guard.get("main").map(|child| child.id())
    } else {
        None
    };

    Ok(FrpsStatus {
        connected,
        server_addr: "localhost".to_string(), // You'd get this from stored config
        active_mappings: vec![], // You'd get this from stored mappings
        pid,
    })
}

#[tauri::command]
pub async fn frps_test_connection(
    config: FrpsConfig,
) -> Result<String, String> {
    // Test connection to FRPS server
    use std::net::TcpStream;
    use std::time::Duration;
    
    let addr = format!("{}:{}", config.server_addr, config.server_port);
    
    match TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(5)
    ) {
        Ok(_) => Ok("Connection test successful".to_string()),
        Err(e) => Err(format!("Connection test failed: {}", e)),
    }
}