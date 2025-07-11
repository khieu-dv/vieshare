use std::{fs::File, io::BufReader};

use crate::logger::logger::IPCLogger;
use crate::utils::{directory::create_directory, file::file_exists, net::download_file_async};

#[cfg(target_os = "windows")]
pub const FRPC_PATH: &str = "VieShare/bin/frpc.exe";

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const FRPC_PATH: &str = "VieShare/bin/frpc";

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub const FRPC_CONFIG_PATH: &str = "VieShare/bin/frpc.toml";

// Function to clean up the frpc.toml file and remove default proxy configurations
fn clean_frpc_config() -> std::io::Result<()> {
    use std::fs;
    use std::io::Write;
    
    // Default clean configuration without any proxy
    let clean_config = r#"# This is a clean frpc configuration file
# Server configuration will be set by the application

"#;
    
    // Write the clean configuration to the file
    let mut file = File::create(FRPC_CONFIG_PATH)?;
    file.write_all(clean_config.as_bytes())?;
    
    println!("Cleaned frpc configuration file: {}", FRPC_CONFIG_PATH);
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn bootstrap_frpc(logger: &IPCLogger) {
    if file_exists(FRPC_PATH) {
        return;
    }

    let _ = create_directory("VieShare/bin");
    let _ = create_directory("VieShare/_temp");

    logger.log("Downloading frpc...");
    let _ = download_file_async(
        "https://github.com/fatedier/frp/releases/download/v0.63.0/frp_0.63.0_windows_amd64.zip",
        "VieShare/_temp/frpc.zip",
    )
    .await;
    logger.log("Downloaded frpc to: \"VieShare/_temp/frpc.zip\"");

    logger.log("Extracting frpc...");
    let _ = extract_frpc();
    logger.log(&format!("Extracted frpc to: \"{}\"", FRPC_PATH));
    
    // Clean up the configuration file after extraction
    logger.log("Cleaning frpc configuration...");
    if let Err(e) = clean_frpc_config() {
        logger.log(&format!("Warning: Failed to clean frpc config: {}", e));
    } else {
        logger.log("Successfully cleaned frpc configuration");
    }
}

#[cfg(target_os = "windows")]
fn extract_frpc() -> zip::result::ZipResult<()> {
    use std::io::{copy, BufWriter};
    use zip::ZipArchive;

    let filepath: File = File::open("VieShare/_temp/frpc.zip")?;
    let mut zip: ZipArchive<BufReader<File>> = ZipArchive::new(BufReader::new(filepath))?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let filename = match file.name().rsplit('/').next() {
            Some(v) => v,
            _ => continue,
        };

        // Extract both frpc.exe and frpc.toml
        if filename == "frpc.exe" {
            let mut outfile = BufWriter::new(File::create(FRPC_PATH)?);
            copy(&mut file, &mut outfile)?;
        } else if filename == "frpc.toml" {
            let toml_path = "VieShare/bin/frpc.toml";
            let mut outfile = BufWriter::new(File::create(toml_path)?);
            copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn bootstrap_frpc(logger: &IPCLogger) {
    if file_exists(FRPC_PATH) {
        return;
    }

    let _ = create_directory("VieShare/bin");
    let _ = create_directory("VieShare/_temp");

    logger.log("Downloading frpc...");
    let _ = download_file_async(
        "https://github.com/fatedier/frp/releases/latest/download/frp_linux_amd64.tar.gz",
        "VieShare/_temp/frpc.tar.gz",
    )
    .await;
    logger.log("Downloaded frpc to: \"VieShare/_temp/frpc.tar.gz\"");

    logger.log("Extracting frpc...");
    let _ = extract_frpc();
    logger.log(&format!("Extracted frpc to: \"{}\"", FRPC_PATH));
    
    // Clean up the configuration file after extraction
    logger.log("Cleaning frpc configuration...");
    if let Err(e) = clean_frpc_config() {
        logger.log(&format!("Warning: Failed to clean frpc config: {}", e));
    } else {
        logger.log("Successfully cleaned frpc configuration");
    }
}

#[cfg(target_os = "macos")]
pub async fn bootstrap_frpc(logger: &IPCLogger) {
    if file_exists(FRPC_PATH) {
        return;
    }

    let _ = create_directory("VieShare/bin");
    let _ = create_directory("VieShare/_temp");

    logger.log("Downloading frpc...");
    
    // Detect macOS architecture
    let arch = std::env::consts::ARCH;
    let download_url = match arch {
        "aarch64" => "https://github.com/fatedier/frp/releases/download/v0.63.0/frp_0.63.0_darwin_arm64.tar.gz",
        "x86_64" => "https://github.com/fatedier/frp/releases/download/v0.63.0/frp_0.63.0_darwin_amd64.tar.gz",
        _ => "https://github.com/fatedier/frp/releases/download/v0.63.0/frp_0.63.0_darwin_amd64.tar.gz", // Default to amd64
    };
    
    let _ = download_file_async(
        download_url,
        "VieShare/_temp/frpc.tar.gz",
    )
    .await;
    logger.log("Downloaded frpc to: \"VieShare/_temp/frpc.tar.gz\"");

    logger.log("Extracting frpc...");
    let _ = extract_frpc();
    logger.log(&format!("Extracted frpc to: \"{}\"", FRPC_PATH));
    
    // Clean up the configuration file after extraction
    logger.log("Cleaning frpc configuration...");
    if let Err(e) = clean_frpc_config() {
        logger.log(&format!("Warning: Failed to clean frpc config: {}", e));
    } else {
        logger.log("Successfully cleaned frpc configuration");
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn extract_frpc() -> std::io::Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    use crate::utils::{file::copy_file};
    
    // For macOS, we need to handle permissions differently
    #[cfg(target_os = "linux")]
    use crate::utils::linux::linux_permit_file;
    
    #[cfg(target_os = "macos")]
    fn macos_permit_file(path: &str, mode: u32) {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        
        if let Ok(metadata) = fs::metadata(path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(mode);
            let _ = fs::set_permissions(path, permissions);
        }
    }

    let file = File::open("VieShare/_temp/frpc.tar.gz")?;
    let decompressor = GzDecoder::new(BufReader::new(file));
    let mut archive = Archive::new(decompressor);

    archive.unpack("VieShare/_temp/frpc")?;

    // Copy from extracted path to binary dir (this path may vary depending on frp version)
    let inner_bin_path = std::fs::read_dir("VieShare/_temp/frpc")?
        .filter_map(Result::ok)
        .find(|entry| entry.path().join("frpc").exists())
        .map(|entry| entry.path().join("frpc"))
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "frpc not found in archive")
        })?;

    let _ = copy_file(inner_bin_path.to_str().unwrap(), FRPC_PATH);
    
    // Set executable permissions based on OS
    #[cfg(target_os = "linux")]
    linux_permit_file(FRPC_PATH, 0o755);
    
    #[cfg(target_os = "macos")]
    macos_permit_file(FRPC_PATH, 0o755);

    // Also copy the frpc.toml file if it exists
    let config_path = std::fs::read_dir("VieShare/_temp/frpc")?
        .filter_map(Result::ok)
        .find(|entry| entry.path().join("frpc.toml").exists())
        .map(|entry| entry.path().join("frpc.toml"));
    
    if let Some(config_source) = config_path {
        if let Some(config_source_str) = config_source.to_str() {
            let _ = copy_file(config_source_str, FRPC_CONFIG_PATH);
        }
    }

    Ok(())
}