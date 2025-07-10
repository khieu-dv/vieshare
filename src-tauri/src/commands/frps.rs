use crate::bin::FRPC_PATH;
use rand::Rng;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::State;

#[cfg(target_os = "windows")]
const FRPC_CONFIG_PATH: &str = "VieShare/bin/frpc.toml";
#[cfg(target_os = "linux")]
const FRPC_CONFIG_PATH: &str = "VieShare/bin/frpc.toml";

// API endpoint for port allocation
const PORT_API_BASE_URL: &str = "http://64.23.133.199:5000/api/v1/ports";

// Hard-coded default values
const DEFAULT_SERVER_ADDR: &str = "64.23.133.199";
const DEFAULT_SERVER_PORT: u16 = 7000;
const DEFAULT_PROTOCOL: &str = "tcp";
const DEFAULT_LOCAL_IP: &str = "127.0.0.1";

// Port range for random allocation
const MIN_PORT: u16 = 8001;
const MAX_PORT: u16 = 8999;

// Maximum number of port mappings per user
const MAX_PORT_MAPPINGS: usize = 3;

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
    pub local_ip: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String,
    pub custom_domains: Option<Vec<String>>,
    pub subdomain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePortMapping {
    pub local_port: u16,
    // Remove remote_port since it will be auto-allocated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpsStatus {
    pub connected: bool,
    pub server_addr: String,
    pub active_mappings: Vec<PortMapping>,
    pub pid: Option<u32>,
    pub max_mappings: usize,
    pub remaining_mappings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub token: String,
    pub user: String,
    pub mappings: HashMap<String, PortMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReleasePortRequest {
    remote_port: u16,
}

// Structures for checking allocated ports
#[derive(Debug, Serialize, Deserialize)]
struct AllocatedPortInfo {
    proxy_name: String,
    remote_port: u16,
    local_ip: String,
    status: String,
    client_version: String,
    today_traffic_in: i64,
    today_traffic_out: i64,
    current_conns: i32,
    last_start_time: String,
    last_close_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AllocatedPortsResponse {
    status: bool,
    message: String,
    allocated_ports: Option<Vec<AllocatedPortInfo>>,
    count: usize,
}

pub type FrpsProcesses = Mutex<HashMap<String, Child>>;
pub type FrpsConfigState = Mutex<Option<Box<dyn Any + Send + Sync>>>;

pub fn init_frps_processes() -> FrpsProcesses {
    Mutex::new(HashMap::new())
}

pub fn init_frps_config_state() -> FrpsConfigState {
    Mutex::new(None)
}

impl TomlConfig {
    fn to_toml_string(&self) -> String {
        let mut config = format!(
            r#"serverAddr = "{}"
serverPort = {}
"#,
            self.server_addr, self.server_port
        );

        if !self.token.is_empty() {
            config.push_str(&format!("auth.token = \"{}\"\n", self.token));
        }
        if !self.user.is_empty() {
            config.push_str(&format!("user = \"{}\"\n", self.user));
        }

        config.push_str("\n");

        for (name, mapping) in &self.mappings {
            config.push_str("[[proxies]]\n");
            config.push_str(&format!("name = \"{}\"\n", name));
            config.push_str(&format!("type = \"{}\"\n", mapping.protocol));
            config.push_str(&format!("localIP = \"{}\"\n", mapping.local_ip));
            config.push_str(&format!("localPort = {}\n", mapping.local_port));
            config.push_str(&format!("remotePort = {}\n", mapping.remote_port));
            config.push_str("\n");
        }

        config
    }
}

// Helper function to check if port mapping limit is reached
fn check_port_mapping_limit(mappings: &HashMap<String, PortMapping>) -> Result<(), String> {
    if mappings.len() >= MAX_PORT_MAPPINGS {
        return Err(format!(
            "Maximum number of port mappings ({}) reached. Please remove existing mappings before adding new ones.",
            MAX_PORT_MAPPINGS
        ));
    }
    Ok(())
}

// Helper function to calculate remaining mappings
fn calculate_remaining_mappings(mappings: &HashMap<String, PortMapping>) -> usize {
    MAX_PORT_MAPPINGS.saturating_sub(mappings.len())
}

// Kill existing frpc processes
#[cfg(target_os = "windows")]
fn kill_existing_frpc_processes() -> Result<(), String> {
    let output = Command::new("taskkill")
        .args(&["/F", "/IM", "frpc.exe"])
        .output()
        .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

    if output.status.success() {
        println!("Successfully killed existing frpc processes");
    } else {
        // It's OK if no processes were found to kill
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("not found") {
            println!("Warning: {}", stderr);
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn kill_existing_frpc_processes() -> Result<(), String> {
    let output = Command::new("pkill")
        .args(&["-f", "frpc"])
        .output()
        .map_err(|e| format!("Failed to execute pkill: {}", e))?;

    if output.status.success() {
        println!("Successfully killed existing frpc processes");
    } else {
        // It's OK if no processes were found to kill
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("no process found") {
            println!("Warning: {}", stderr);
        }
    }
    Ok(())
}

// Wait for processes to be killed
fn wait_for_process_cleanup() {
    std::thread::sleep(std::time::Duration::from_millis(1000));
}

// Check if frpc process is running
#[cfg(target_os = "windows")]
fn is_frpc_running() -> bool {
    let output = Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq frpc.exe"])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("frpc.exe")
        }
        Err(_) => false,
    }
}

#[cfg(target_os = "linux")]
fn is_frpc_running() -> bool {
    let output = Command::new("pgrep").args(&["-f", "frpc"]).output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

// Get allocated ports from API
async fn get_allocated_ports() -> Result<Vec<u16>, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/allocated", PORT_API_BASE_URL))
        .send()
        .await
        .map_err(|e| format!("Failed to get allocated ports: {}", e))?;

    if response.status().is_success() {
        let result: AllocatedPortsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse allocated ports response: {}", e))?;

        // Handle the case where allocated_ports is null
        let allocated_ports: Vec<u16> = result
            .allocated_ports
            .unwrap_or_default() // If null, use empty Vec
            .iter()
            .map(|port_info| port_info.remote_port)
            .collect();

        Ok(allocated_ports)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("API error: {}", error_text))
    }
}

// Find available port in range 8000-9000
// Find available port in range 8000-8010, excluding restricted ports
async fn find_available_port_in_range() -> Result<u16, String> {
    let restricted_ports = vec![8081, 8090, 9000];
    let allocated_ports = get_allocated_ports().await?;

    // Create a HashSet for O(1) lookup
    let allocated_set: std::collections::HashSet<u16> = allocated_ports.into_iter().collect();
    let restricted_set: std::collections::HashSet<u16> = restricted_ports.into_iter().collect();

    // Collect all available ports in the range
    let mut available_ports: Vec<u16> = Vec::new();
    for port in MIN_PORT..=MAX_PORT {
        if !allocated_set.contains(&port) && !restricted_set.contains(&port) {
            available_ports.push(port);
        }
    }

    if available_ports.is_empty() {
        return Err(format!(
            "No available ports in range {}-{} after excluding restricted ports",
            MIN_PORT, MAX_PORT
        ));
    }

    // Randomly select an available port
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..available_ports.len());
    let selected_port = available_ports[random_index];

    Ok(selected_port)
}

// Simplified port allocation - just find available port without API call
async fn allocate_port_locally() -> Result<u16, String> {
    find_available_port_in_range().await
}

// Generate unique mapping name
fn generate_mapping_name(
    remote_port: u16,
    existing_mappings: &HashMap<String, PortMapping>,
) -> String {
    let base_name = format!("nextjs{}", remote_port);
    let mut counter = 1;
    let mut name = base_name.clone();

    while existing_mappings.contains_key(&name) {
        name = format!("{}{}", base_name, counter);
        counter += 1;
    }

    name
}

fn load_config_from_toml() -> Result<TomlConfig, String> {
    if !Path::new(FRPC_CONFIG_PATH).exists() {
        return Ok(TomlConfig {
            server_addr: DEFAULT_SERVER_ADDR.to_string(),
            server_port: DEFAULT_SERVER_PORT,
            token: "".to_string(),
            user: "".to_string(),
            mappings: HashMap::new(),
        });
    }

    let content = fs::read_to_string(FRPC_CONFIG_PATH)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config = TomlConfig {
        server_addr: DEFAULT_SERVER_ADDR.to_string(),
        server_port: DEFAULT_SERVER_PORT,
        token: "".to_string(),
        user: "".to_string(),
        mappings: HashMap::new(),
    };

    let mut current_proxy: Option<PortMapping> = None;
    let mut current_proxy_name = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("serverAddr") {
            if let Some(value) = line.split('=').nth(1) {
                config.server_addr = value.trim().trim_matches('"').to_string();
            }
        } else if line.starts_with("serverPort") {
            if let Some(value) = line.split('=').nth(1) {
                config.server_port = value.trim().parse().unwrap_or(DEFAULT_SERVER_PORT);
            }
        } else if line.starts_with("auth.token") {
            if let Some(value) = line.split('=').nth(1) {
                config.token = value.trim().trim_matches('"').to_string();
            }
        } else if line.starts_with("user") {
            if let Some(value) = line.split('=').nth(1) {
                config.user = value.trim().trim_matches('"').to_string();
            }
        } else if line.starts_with("[[proxies]]") {
            if let Some(proxy) = current_proxy.take() {
                config.mappings.insert(current_proxy_name.clone(), proxy);
            }
            current_proxy = Some(PortMapping {
                name: String::new(),
                local_ip: DEFAULT_LOCAL_IP.to_string(),
                local_port: 0,
                remote_port: 0,
                protocol: DEFAULT_PROTOCOL.to_string(),
                custom_domains: None,
                subdomain: None,
            });
        } else if let Some(ref mut proxy) = current_proxy {
            if line.starts_with("name") {
                if let Some(value) = line.split('=').nth(1) {
                    let name = value.trim().trim_matches('"').to_string();
                    proxy.name = name.clone();
                    current_proxy_name = name;
                }
            } else if line.starts_with("type") {
                if let Some(value) = line.split('=').nth(1) {
                    proxy.protocol = value.trim().trim_matches('"').to_string();
                }
            } else if line.starts_with("localIP") {
                if let Some(value) = line.split('=').nth(1) {
                    proxy.local_ip = value.trim().trim_matches('"').to_string();
                }
            } else if line.starts_with("localPort") {
                if let Some(value) = line.split('=').nth(1) {
                    proxy.local_port = value.trim().parse().unwrap_or(0);
                }
            } else if line.starts_with("remotePort") {
                if let Some(value) = line.split('=').nth(1) {
                    proxy.remote_port = value.trim().parse().unwrap_or(0);
                }
            } else if line.starts_with("subdomain") {
                if let Some(value) = line.split('=').nth(1) {
                    proxy.subdomain = Some(value.trim().trim_matches('"').to_string());
                }
            } else if line.starts_with("customDomains") {
                if let Some(value) = line.split('=').nth(1) {
                    let domains = value
                        .trim()
                        .trim_matches('[')
                        .trim_matches(']')
                        .split(',')
                        .map(|d| d.trim().trim_matches('"').to_string())
                        .filter(|d| !d.is_empty())
                        .collect::<Vec<String>>();
                    if !domains.is_empty() {
                        proxy.custom_domains = Some(domains);
                    }
                }
            }
        }
    }

    if let Some(proxy) = current_proxy {
        config.mappings.insert(current_proxy_name, proxy);
    }

    Ok(config)
}

fn save_config_to_toml(config: &TomlConfig) -> Result<(), String> {
    if let Some(parent) = Path::new(FRPC_CONFIG_PATH).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config_content = config.to_toml_string();
    fs::write(FRPC_CONFIG_PATH, config_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn frps_connect(
    processes: State<'_, FrpsProcesses>,
    config_state: State<'_, FrpsConfigState>,
) -> Result<String, String> {
    let mut processes_guard = processes.lock().map_err(|e| e.to_string())?;

    // Check if already connected
    if processes_guard.contains_key("main") {
        return Err("FRPS client is already running".to_string());
    }

    // Kill any existing frpc processes to prevent conflicts
    println!("Checking for existing frpc processes...");
    if is_frpc_running() {
        println!("Found existing frpc processes, killing them...");
        kill_existing_frpc_processes()?;
        wait_for_process_cleanup();
    }

    let mut toml_config = load_config_from_toml()?;

    // Use hard-coded values
    toml_config.server_addr = DEFAULT_SERVER_ADDR.to_string();
    toml_config.server_port = DEFAULT_SERVER_PORT;
    toml_config.token = "".to_string();
    toml_config.user = "".to_string();

    save_config_to_toml(&toml_config)?;

    {
        let mut state_guard = config_state.lock().map_err(|e| e.to_string())?;
        *state_guard = Some(Box::new(toml_config));
    }

    // Start new frpc process
    let child = Command::new(FRPC_PATH)
        .arg("-c")
        .arg(FRPC_CONFIG_PATH)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start frpc: {}", e))?;

    processes_guard.insert("main".to_string(), child);

    Ok("FRPS client connected successfully".to_string())
}

#[tauri::command]
pub async fn frps_disconnect(processes: State<'_, FrpsProcesses>) -> Result<String, String> {
    let mut processes_guard = processes.lock().map_err(|e| e.to_string())?;

    if let Some(mut child) = processes_guard.remove("main") {
        child
            .kill()
            .map_err(|e| format!("Failed to kill process: {}", e))?;

        // Also kill any other frpc processes that might be running
        if is_frpc_running() {
            kill_existing_frpc_processes()?;
        }

        Ok("FRPS client disconnected successfully".to_string())
    } else {
        // Even if we don't have a tracked process, kill any running frpc
        if is_frpc_running() {
            kill_existing_frpc_processes()?;
            Ok("FRPS client disconnected successfully (cleaned up orphaned processes)".to_string())
        } else {
            Err("No active FRPS connection found".to_string())
        }
    }
}

// Updated frps_add_port_mapping function call with port limit check
#[tauri::command]
pub async fn frps_add_port_mapping(
    simple_mapping: SimplePortMapping,
    processes: State<'_, FrpsProcesses>,
    config_state: State<'_, FrpsConfigState>,
) -> Result<String, String> {
    let mut toml_config = load_config_from_toml()?;

    // Check if port mapping limit is reached
    check_port_mapping_limit(&toml_config.mappings)?;

    // Get remote port locally without API call
    let remote_port = allocate_port_locally().await?;

    // Generate unique name using remote_port instead of local_port
    let mapping_name = generate_mapping_name(remote_port, &toml_config.mappings);

    // Create full mapping with hard-coded values
    let new_mapping = PortMapping {
        name: mapping_name.clone(),
        local_ip: DEFAULT_LOCAL_IP.to_string(),
        local_port: simple_mapping.local_port,
        remote_port,
        protocol: DEFAULT_PROTOCOL.to_string(),
        custom_domains: None,
        subdomain: None,
    };

    toml_config
        .mappings
        .insert(mapping_name.clone(), new_mapping);

    save_config_to_toml(&toml_config)?;

    {
        let mut state_guard = config_state.lock().map_err(|e| e.to_string())?;
        *state_guard = Some(Box::new(toml_config));
    }

    // Restart frpc if it's running
    let mut processes_guard = processes.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = processes_guard.remove("main") {
        child
            .kill()
            .map_err(|e| format!("Failed to kill process: {}", e))?;

        // Wait a bit and clean up any remaining processes
        wait_for_process_cleanup();
        if is_frpc_running() {
            kill_existing_frpc_processes()?;
            wait_for_process_cleanup();
        }

        let new_child = Command::new(FRPC_PATH)
            .arg("-c")
            .arg(FRPC_CONFIG_PATH)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to restart frpc: {}", e))?;

        processes_guard.insert("main".to_string(), new_child);
    }

    Ok(format!(
        "Port mapping {} added successfully (Local: {} → Remote: {} from range {}-{})",
        mapping_name, simple_mapping.local_port, remote_port, MIN_PORT, MAX_PORT
    ))
}

#[tauri::command]
pub async fn frps_remove_port_mapping(
    mapping_name: String,
    processes: State<'_, FrpsProcesses>,
    config_state: State<'_, FrpsConfigState>,
) -> Result<String, String> {
    let mut toml_config = load_config_from_toml()?;

    // Get the mapping to release the port
    let mapping = toml_config
        .mappings
        .get(&mapping_name)
        .ok_or_else(|| "Port mapping not found".to_string())?;

    let remote_port = mapping.remote_port;

    // Remove from config
    toml_config.mappings.remove(&mapping_name);

    save_config_to_toml(&toml_config)?;

    {
        let mut state_guard = config_state.lock().map_err(|e| e.to_string())?;
        *state_guard = Some(Box::new(toml_config));
    }

    // Restart frpc if it's running
    let mut processes_guard = processes.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = processes_guard.remove("main") {
        child
            .kill()
            .map_err(|e| format!("Failed to kill process: {}", e))?;

        // Wait a bit and clean up any remaining processes
        wait_for_process_cleanup();
        if is_frpc_running() {
            kill_existing_frpc_processes()?;
            wait_for_process_cleanup();
        }

        let new_child = Command::new(FRPC_PATH)
            .arg("-c")
            .arg(FRPC_CONFIG_PATH)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to restart frpc: {}", e))?;

        processes_guard.insert("main".to_string(), new_child);
    }

    Ok(format!(
        "Port mapping {} removed successfully",
        mapping_name
    ))
}

#[tauri::command]
pub async fn frps_get_status(
    processes: State<'_, FrpsProcesses>,
    config_state: State<'_, FrpsConfigState>,
) -> Result<FrpsStatus, String> {
    let processes_guard = processes.lock().map_err(|e| e.to_string())?;
    let config_guard = config_state.lock().map_err(|e| e.to_string())?;

    // Check both our tracked process and system processes
    let tracked_connected = processes_guard.contains_key("main");
    let system_connected = is_frpc_running();
    let connected = tracked_connected || system_connected;

    let pid = if tracked_connected {
        processes_guard.get("main").map(|child| child.id())
    } else {
        None
    };

    let (server_addr, active_mappings) = if let Some(config_any) = config_guard.as_ref() {
        if let Some(config) = config_any.downcast_ref::<TomlConfig>() {
            (
                config.server_addr.clone(),
                config.mappings.values().cloned().collect(),
            )
        } else {
            (DEFAULT_SERVER_ADDR.to_string(), vec![])
        }
    } else {
        (DEFAULT_SERVER_ADDR.to_string(), vec![])
    };

    let remaining_mappings = calculate_remaining_mappings(
        &active_mappings.iter().map(|m| (m.name.clone(), m.clone())).collect()
    );

    Ok(FrpsStatus {
        connected,
        server_addr,
        active_mappings,
        pid,
        max_mappings: MAX_PORT_MAPPINGS,
        remaining_mappings,
    })
}

#[tauri::command]
pub async fn frps_test_connection() -> Result<String, String> {
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = format!("{}:{}", DEFAULT_SERVER_ADDR, DEFAULT_SERVER_PORT);

    match TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(5),
    ) {
        Ok(_) => Ok("Connection test successful".to_string()),
        Err(e) => Err(format!("Connection test failed: {}", e)),
    }
}

#[tauri::command]
pub async fn frps_load_config() -> Result<FrpsConfig, String> {
    Ok(FrpsConfig {
        server_addr: DEFAULT_SERVER_ADDR.to_string(),
        server_port: DEFAULT_SERVER_PORT,
        token: "".to_string(),
        user: "".to_string(),
    })
}

#[tauri::command]
pub async fn frps_get_mappings() -> Result<Vec<PortMapping>, String> {
    let toml_config = load_config_from_toml()?;
    Ok(toml_config.mappings.values().cloned().collect())
}

// New command to get port mapping limits
#[tauri::command]
pub async fn frps_get_port_limits() -> Result<(usize, usize), String> {
    let toml_config = load_config_from_toml()?;
    let remaining = calculate_remaining_mappings(&toml_config.mappings);
    Ok((MAX_PORT_MAPPINGS, remaining))
}

// Clean up function to be called on app shutdown
#[tauri::command]
pub async fn frps_cleanup() -> Result<String, String> {
    if is_frpc_running() {
        kill_existing_frpc_processes()?;
        Ok("Cleaned up all frpc processes".to_string())
    } else {
        Ok("No frpc processes found to clean up".to_string())
    }
}