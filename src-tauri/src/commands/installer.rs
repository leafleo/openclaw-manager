use crate::utils::{log_sanitizer, platform, shell};
use serde::{Deserialize, Serialize};
use tauri::command;
use log::{info, warn, error, debug};
use std::path::Path;
use std::fs;

/// Environment check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStatus {
    /// Whether Node.js is installed
    pub node_installed: bool,
    /// Node.js version
    pub node_version: Option<String>,
    /// Whether Node.js version meets requirement (>=22)
    pub node_version_ok: bool,
    /// Whether Git is installed
    pub git_installed: bool,
    /// Git version
    pub git_version: Option<String>,
    /// Whether OpenClaw is installed
    pub openclaw_installed: bool,
    /// OpenClaw version
    pub openclaw_version: Option<String>,
    /// Whether gateway service is installed
    pub gateway_service_installed: bool,
    /// Whether config directory exists
    pub config_dir_exists: bool,
    /// Whether everything is ready
    pub ready: bool,
    /// Operating system
    pub os: String,
}

/// Installation progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub step: String,
    pub progress: u8,
    pub message: String,
    pub error: Option<String>,
}

/// Installation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// Check environment status
#[command]
pub async fn check_environment() -> Result<EnvironmentStatus, String> {
    info!("[Environment Check] Starting system environment check...");

    let os = platform::get_os();
    info!("[Environment Check] Operating system: {}", os);

    // Run expensive checks concurrently
    info!("[Environment Check] Checking Node.js, Git, and OpenClaw concurrently...");
    let (node_res, git_res, openclaw_res) = tokio::join!(
        tokio::task::spawn_blocking(|| get_node_version()),
        tokio::task::spawn_blocking(|| get_git_version()),
        tokio::task::spawn_blocking(|| get_openclaw_version())
    );

    let node_version = node_res.unwrap_or(None);
    let git_version = git_res.unwrap_or(None);
    let openclaw_version = openclaw_res.unwrap_or(None);

    let node_installed = node_version.is_some();
    let node_version_ok = check_node_version_requirement(&node_version);
    info!("[Environment Check] Node.js: installed={}, version={:?}, version_ok={}",
        node_installed, node_version, node_version_ok);

    let git_installed = git_version.is_some();
    info!("[Environment Check] Git: installed={}, version={:?}",
        git_installed, git_version);

    let openclaw_installed = openclaw_version.is_some();
    info!("[Environment Check] OpenClaw: installed={}, version={:?}",
        openclaw_installed, openclaw_version);

    // Check Gateway Service (only if OpenClaw is installed)
    let gateway_service_installed = if openclaw_installed {
        info!("[Environment Check] Checking Gateway Service...");
        let installed = tokio::task::spawn_blocking(|| check_gateway_installed()).await.unwrap_or(false);
        info!("[Environment Check] Gateway Service: installed={}", installed);
        installed
    } else {
        false
    };

    // Check config directory
    let config_dir = platform::get_config_dir();
    let config_dir_exists = std::path::Path::new(&config_dir).exists();
    info!("[Environment Check] Config directory: {}, exists={}", config_dir, config_dir_exists);

    let ready = node_installed && node_version_ok && openclaw_installed && gateway_service_installed;
    info!("[Environment Check] Environment ready status: ready={}", ready);
    
    Ok(EnvironmentStatus {
        node_installed,
        node_version,
        node_version_ok,
        git_installed,
        git_version,
        openclaw_installed,
        openclaw_version,
        gateway_service_installed,
        config_dir_exists,
        ready,
        os,
    })
}

/// Get Node.js version
fn get_node_version() -> Option<String> {
    if platform::is_windows() {
        // Windows: First try direct call (if PATH is updated)
        if let Ok(v) = shell::run_cmd_output("node --version") {
            let version = v.trim().to_string();
            if !version.is_empty() && version.starts_with('v') {
                info!("[Environment Check] Found Node.js via PATH: {}", version);
                return Some(version);
            }
        }

        // Windows: Check common installation paths
        let possible_paths = get_windows_node_paths();
        for path in possible_paths {
            if std::path::Path::new(&path).exists() {
                let cmd = format!("\"{}\" --version", path);
                if let Ok(output) = shell::run_cmd_output(&cmd) {
                    let version = output.trim().to_string();
                    if !version.is_empty() && version.starts_with('v') {
                        info!("[Environment Check] Found Node.js at {}: {}", path, version);
                        return Some(version);
                    }
                }
            }
        }

        None
    } else {
        // First try direct call
        if let Ok(v) = shell::run_command_output("node", &["--version"]) {
            return Some(v.trim().to_string());
        }

        // Detect common Node.js installation paths (macOS/Linux)
        let possible_paths = get_unix_node_paths();
        for path in possible_paths {
            if std::path::Path::new(&path).exists() {
                if let Ok(output) = shell::run_command_output(&path, &["--version"]) {
                    info!("[Environment Check] Found Node.js at {}: {}", path, output.trim());
                    return Some(output.trim().to_string());
                }
            }
        }

        // Try to detect by loading user environment via shell
        if let Ok(output) = shell::run_bash_output("source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; node --version 2>/dev/null") {
            if !output.is_empty() && output.starts_with('v') {
                info!("[Environment Check] Found Node.js via user shell: {}", output.trim());
                return Some(output.trim().to_string());
            }
        }

        None
    }
}

/// Get possible Node.js paths on Unix systems
fn get_unix_node_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // Homebrew (macOS)
    paths.push("/opt/homebrew/bin/node".to_string());
    paths.push("/usr/local/bin/node".to_string());

    // System installation
    paths.push("/usr/bin/node".to_string());

    // nvm (check common versions)
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();

        // nvm default versions
        paths.push(format!("{}/.nvm/versions/node/v22.0.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.1.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.2.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.11.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.12.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v23.0.0/bin/node", home_str));

        // Try nvm alias default
        let nvm_default = format!("{}/.nvm/alias/default", home_str);
        if let Ok(version) = std::fs::read_to_string(&nvm_default) {
            let version = version.trim();
            if !version.is_empty() {
                paths.insert(0, format!("{}/.nvm/versions/node/v{}/bin/node", home_str, version));
            }
        }

        // fnm
        paths.push(format!("{}/.fnm/aliases/default/bin/node", home_str));

        // volta
        paths.push(format!("{}/.volta/bin/node", home_str));

        // asdf
        paths.push(format!("{}/.asdf/shims/node", home_str));

        // mise
        paths.push(format!("{}/.local/share/mise/shims/node", home_str));
    }

    paths
}

/// Get possible Node.js paths on Windows systems
fn get_windows_node_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // 1. Standard installation paths (Program Files)
    paths.push("C:\\Program Files\\nodejs\\node.exe".to_string());
    paths.push("C:\\Program Files (x86)\\nodejs\\node.exe".to_string());

    // 2. nvm for Windows
    paths.push("C:\\nvm4w\\nodejs\\node.exe".to_string());

    // 3. Various installations in user directory
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();

        // nvm for Windows user installation
        paths.push(format!("{}\\AppData\\Roaming\\nvm\\current\\node.exe", home_str));

        // fnm
        paths.push(format!("{}\\AppData\\Roaming\\fnm\\aliases\\default\\node.exe", home_str));
        paths.push(format!("{}\\AppData\\Local\\fnm\\aliases\\default\\node.exe", home_str));
        paths.push(format!("{}\\.fnm\\aliases\\default\\node.exe", home_str));

        // volta
        paths.push(format!("{}\\AppData\\Local\\Volta\\bin\\node.exe", home_str));

        // scoop
        paths.push(format!("{}\\scoop\\apps\\nodejs\\current\\node.exe", home_str));
        paths.push(format!("{}\\scoop\\apps\\nodejs-lts\\current\\node.exe", home_str));

        // chocolatey
        paths.push("C:\\ProgramData\\chocolatey\\lib\\nodejs\\tools\\node.exe".to_string());
    }

    // 4. Installation paths from registry
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        paths.push(format!("{}\\nodejs\\node.exe", program_files));
    }
    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        paths.push(format!("{}\\nodejs\\node.exe", program_files_x86));
    }

    // 5. nvm-windows symlink path
    if let Ok(nvm_symlink) = std::env::var("NVM_SYMLINK") {
        paths.insert(0, format!("{}\\node.exe", nvm_symlink));
    }

    // 6. Current version under nvm-windows NVM_HOME path
    if let Ok(nvm_home) = std::env::var("NVM_HOME") {
        let settings_path = format!("{}\\settings.txt", nvm_home);
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            for line in content.lines() {
                if line.starts_with("current:") {
                    if let Some(version) = line.strip_prefix("current:") {
                        let version = version.trim();
                        if !version.is_empty() {
                            paths.insert(0, format!("{}\\v{}\\node.exe", nvm_home, version));
                        }
                    }
                }
            }
        }
    }

    paths
}

/// Get Git version
fn get_git_version() -> Option<String> {
    if platform::is_windows() {
        if let Ok(v) = shell::run_cmd_output("git --version") {
            let version = v.trim().to_string();
            if !version.is_empty() && version.contains("git version") {
                let ver = version.replace("git version ", "");
                let ver = ver.split('.').take(3).collect::<Vec<_>>().join(".");
                return Some(ver);
            }
        }
        None
    } else {
        if let Ok(v) = shell::run_command_output("git", &["--version"]) {
            let version = v.trim().to_string();
            if !version.is_empty() && version.contains("git version") {
                let ver = version.replace("git version ", "");
                return Some(ver.trim().to_string());
            }
        }
        None
    }
}

/// Get OpenClaw version
fn get_openclaw_version() -> Option<String> {
    shell::run_openclaw(&["--version"])
        .ok()
        .map(|v| v.trim().to_string())
}

/// Check if Node.js version is >= 22
fn check_node_version_requirement(version: &Option<String>) -> bool {
    if let Some(v) = version {
        let major = v.trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        major >= 22
    } else {
        false
    }
}

/// Check if gateway service is installed
fn check_gateway_installed() -> bool {
    match shell::run_openclaw(&["gateway", "status"]) {
        Ok(output) => {
            let lower = output.to_lowercase();
            if lower.contains("not installed") || lower.contains("not found") {
                return false;
            }
            true
        }
        Err(e) => {
            let lower = e.to_lowercase();
            if lower.contains("not installed") || lower.contains("not found") {
                return false;
            }
            debug!("[Environment Check] Gateway status check failed: {}", e);
            false
        }
    }
}

/// Install gateway service (opens elevated terminal)
#[command]
pub async fn install_gateway_service() -> Result<String, String> {
    info!("[Gateway Install] Starting gateway service installation...");
    let os = platform::get_os();
    info!("[Gateway Install] Detected operating system: {}", os);

    match os.as_str() {
        "windows" => install_gateway_windows().await,
        "macos" => install_gateway_macos().await,
        "linux" => install_gateway_linux().await,
        _ => Err(format!("Unsupported operating system: {}", os)),
    }
}

/// Install gateway service on Windows
async fn install_gateway_windows() -> Result<String, String> {
    info!("[Gateway Install] Opening elevated PowerShell for gateway install...");

    let openclaw_path = shell::get_openclaw_path().unwrap_or_else(|| "openclaw".to_string());
    let escaped_path = openclaw_path.replace('\\', "\\\\");

    let script = format!(r#"
Start-Process powershell -ArgumentList '-NoExit', '-Command', '
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  OpenClaw Gateway Service Installer" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Installing OpenClaw Gateway as a system service..." -ForegroundColor Yellow
Write-Host ""

try {{
    & "{}" gateway install
    Write-Host ""
    Write-Host "Gateway service installed successfully!" -ForegroundColor Green
}} catch {{
    Write-Host "Installation failed: $_" -ForegroundColor Red
}}

Write-Host ""
Write-Host "You can close this window and click Refresh in OpenClaw Manager." -ForegroundColor Cyan
Write-Host ""
Read-Host "Press Enter to close this window"
' -Verb RunAs
"#, escaped_path);

    match shell::run_powershell_output(&script) {
        Ok(_) => {
            info!("[Gateway Install] Elevated terminal launched successfully");
            Ok("Gateway install terminal opened with administrator privileges. Please complete the installation and click Refresh.".to_string())
        }
        Err(e) => {
            warn!("[Gateway Install] Failed to launch elevated terminal: {}", e);
            Err(format!("Failed to open administrator terminal: {}. Please open PowerShell as Administrator and run: openclaw gateway install", e))
        }
    }
}

/// Install gateway service on macOS
async fn install_gateway_macos() -> Result<String, String> {
    info!("[Gateway Install] Opening terminal for gateway install on macOS...");

    let script_content = r#"#!/bin/bash
clear
echo "========================================"
echo "  OpenClaw Gateway Service Installer"
echo "========================================"
echo ""
echo "Installing OpenClaw Gateway as a system service..."
echo "You may be prompted for your password."
echo ""

sudo openclaw gateway install

echo ""
if [ $? -eq 0 ]; then
    echo "✅ Gateway service installed successfully!"
else
    echo "❌ Installation failed. Please check the error above."
fi
echo ""
echo "You can close this window and click Refresh in OpenClaw Manager."
read -p "Press Enter to close this window..."
"#;

    let script_path = "/tmp/openclaw_gateway_install.command";
    std::fs::write(script_path, script_content)
        .map_err(|e| format!("Failed to create script: {}", e))?;

    std::process::Command::new("chmod")
        .args(["+x", script_path])
        .output()
        .map_err(|e| format!("Failed to set permissions: {}", e))?;

    std::process::Command::new("open")
        .arg(script_path)
        .spawn()
        .map_err(|e| format!("Failed to launch terminal: {}", e))?;

    info!("[Gateway Install] Terminal launched successfully on macOS");
    Ok("Gateway install terminal opened. Please enter your password when prompted and click Refresh after completion.".to_string())
}

/// Install gateway service on Linux
async fn install_gateway_linux() -> Result<String, String> {
    info!("[Gateway Install] Opening terminal for gateway install on Linux...");

    let script_content = r#"#!/bin/bash
clear
echo "========================================"
echo "  OpenClaw Gateway Service Installer"
echo "========================================"
echo ""
echo "Installing OpenClaw Gateway as a system service..."
echo "You may be prompted for your password."
echo ""

sudo openclaw gateway install

echo ""
if [ $? -eq 0 ]; then
    echo "✅ Gateway service installed successfully!"
else
    echo "❌ Installation failed. Please check the error above."
fi
echo ""
echo "You can close this window and click Refresh in OpenClaw Manager."
read -p "Press Enter to close this window..."
"#;

    let script_path = "/tmp/openclaw_gateway_install.sh";
    std::fs::write(script_path, script_content)
        .map_err(|e| format!("Failed to create script: {}", e))?;

    std::process::Command::new("chmod")
        .args(["+x", script_path])
        .output()
        .map_err(|e| format!("Failed to set permissions: {}", e))?;

    let terminals = ["gnome-terminal", "xfce4-terminal", "konsole", "xterm"];
    for term in terminals {
        if std::process::Command::new(term)
            .args(["--", script_path])
            .spawn()
            .is_ok()
        {
            info!("[Gateway Install] Terminal '{}' launched successfully on Linux", term);
            return Ok("Gateway install terminal opened. Please enter your password when prompted and click Refresh after completion.".to_string());
        }
    }

    warn!("[Gateway Install] No terminal emulator found on Linux");
    Err("Unable to launch terminal. Please open a terminal and run: sudo openclaw gateway install".to_string())
}

/// Get runtime bundles directory
fn get_runtime_bundles_dir() -> Result<String, String> {
    // Try to get from current executable path
    let mut runtime_bundles_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    // Navigate to runtime-bundles directory
    runtime_bundles_dir.pop();
    runtime_bundles_dir.pop();
    runtime_bundles_dir.pop();
    runtime_bundles_dir.push("runtime-bundles");
    
    let path = runtime_bundles_dir.to_string_lossy().to_string();
    info!("[Local Bundle Install] Runtime bundles directory: {}", path);
    
    Ok(path)
}

/// Install Node.js from local bundle
#[command]
pub async fn install_nodejs() -> Result<InstallResult, String> {
    info!("[Local Bundle Install] Installing Node.js from local bundle...");
    
    let os = platform::get_os();
    let arch = platform::get_arch();
    info!("[Local Bundle Install] Detected OS: {}, Architecture: {}", os, arch);
    
    let bundles_dir = get_runtime_bundles_dir()?;
    
    if !Path::new(&bundles_dir).exists() {
        return Err(format!("Runtime bundles directory not found: {}", bundles_dir));
    }
    
    let platform_dir = match os.as_str() {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "linux",
        _ => return Err(format!("Unsupported OS: {}", os)),
    };
    
    let node_dir = format!("{}/{}/node", bundles_dir, platform_dir);
    
    let node_file = match (os.as_str(), arch.as_str()) {
        ("windows", "x86_64") => "node-v18.20.4-win-x64.zip",
        ("windows", "x86") => "node-v18.20.4-win-x86.zip",
        ("macos", "aarch64") => "node-v18.20.4-darwin-arm64.tar.gz",
        ("macos", "x86_64") => "node-v18.20.4-darwin-x64.tar.gz",
        ("linux", "x86_64") => "node-v18.20.4-linux-x64.tar.xz",
        _ => return Err(format!("Unsupported OS/arch combination: {}/{}", os, arch)),
    };
    
    let node_file_path = format!("{}/{}", node_dir, node_file);
    info!("[Local Bundle Install] Node.js bundle: {}", node_file_path);
    
    if !Path::new(&node_file_path).exists() {
        return Err(format!("Node.js bundle not found: {}", node_file_path));
    }
    
    // Create installation directory
    let install_dir = format!("{}/runtime/node", bundles_dir);
    if let Err(e) = fs::create_dir_all(&install_dir) {
        return Err(format!("Failed to create installation directory: {}", e));
    }
    
    // Extract the bundle
    info!("[Local Bundle Install] Extracting Node.js bundle...");
    let extract_result = if node_file.ends_with(".zip") {
        if platform::is_windows() {
            let script = format!(r#"
Expand-Archive -Path '{}' -DestinationPath '{}' -Force
"#, node_file_path, install_dir);
            shell::run_powershell_output(&script)
        } else {
            Err("ZIP files are only supported on Windows".to_string())
        }
    } else if node_file.ends_with(".tar.gz") || node_file.ends_with(".tar.xz") {
        let script = format!(r#"
tar -xf '{}' -C '{}'
"#, node_file_path, install_dir);
        shell::run_bash_output(&script)
    } else {
        Err(format!("Unsupported file format: {}", node_file))
    };
    
    if let Err(e) = extract_result {
        return Err(format!("Failed to extract Node.js bundle: {}", e));
    }
    
    // Determine the extracted directory
    let extracted_dir = if node_file.ends_with(".zip") {
        format!("{}/node-v18.20.4-win-x64", install_dir)
    } else if node_file.ends_with(".tar.gz") || node_file.ends_with(".tar.xz") {
        format!("{}/node-v18.20.4-darwin-x64", install_dir)
    } else {
        install_dir
    };
    
    // Configure environment variables
    info!("[Local Bundle Install] Configuring Node.js environment variables...");
    
    if platform::is_windows() {
        let script = format!(r#"
setx NODE_HOME '{}' /M
setx PATH '%NODE_HOME%\bin;%PATH%' /M
"#, extracted_dir);
        if let Err(e) = shell::run_cmd_output(&script) {
            return Err(format!("Failed to set environment variables: {}", e));
        }
    } else {
        let env_lines = format!(r#"export NODE_HOME={}
export PATH=$NODE_HOME/bin:$PATH
"#, extracted_dir);
        
        if let Some(home) = dirs::home_dir() {
            let zprofile_path = format!("{}/.zprofile", home.display());
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&zprofile_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    f.write_all(env_lines.as_bytes())
                });
            
            let bash_profile_path = format!("{}/.bash_profile", home.display());
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&bash_profile_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    f.write_all(env_lines.as_bytes())
                });
        }
    }
    
    Ok(InstallResult {
        success: true,
        message: "Node.js installed successfully from local bundle!".to_string(),
        error: None,
    })
}

/// Install Git from local bundle
#[command]
pub async fn install_git() -> Result<InstallResult, String> {
    info!("[Local Bundle Install] Installing Git from local bundle...");
    
    let os = platform::get_os();
    let arch = platform::get_arch();
    
    let bundles_dir = get_runtime_bundles_dir()?;
    
    let platform_dir = match os.as_str() {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "linux",
        _ => return Err(format!("Unsupported OS: {}", os)),
    };
    
    let git_dir = format!("{}/{}/git", bundles_dir, platform_dir);
    
    let git_file = match os.as_str() {
        "windows" => "PortableGit-2.43.0-64-bit.7z.exe",
        "macos" => "git-2.33.0-intel-universal-mavericks.dmg",
        "linux" => "git-2.43.0.tar.gz",
        _ => return Err(format!("Unsupported OS: {}", os)),
    };
    
    let git_file_path = format!("{}/{}", git_dir, git_file);
    info!("[Local Bundle Install] Git bundle: {}", git_file_path);
    
    if !Path::new(&git_file_path).exists() {
        return Err(format!("Git bundle not found: {}", git_file_path));
    }
    
    let install_dir = format!("{}/runtime/git", bundles_dir);
    if let Err(e) = fs::create_dir_all(&install_dir) {
        return Err(format!("Failed to create installation directory: {}", e));
    }
    
    if platform::is_windows() {
        let script = format!(r#"
'{}' -y -o'{}'
"#, git_file_path, install_dir);
        if let Err(e) = shell::run_cmd_output(&script) {
            return Err(format!("Failed to install Git: {}", e));
        }
        
        let script = format!(r#"
setx GIT_HOME '{}' /M
setx PATH '%GIT_HOME%\bin;%PATH%' /M
"#, install_dir);
        if let Err(e) = shell::run_cmd_output(&script) {
            return Err(format!("Failed to set environment variables: {}", e));
        }
    } else if platform::is_macos() {
        let script = format!(r#"
hdiutil mount '{}'
sudo installer -pkg '/Volumes/Git/Git.pkg' -target /
hdiutil unmount '/Volumes/Git'
"#, git_file_path);
        if let Err(e) = shell::run_bash_output(&script) {
            return Err(format!("Failed to install Git: {}", e));
        }
    } else {
        let script = format!(r#"
tar -xf '{}' -C '{}'
cd '{}/git-2.43.0'
./configure
make
sudo make install
"#, git_file_path, install_dir, install_dir);
        if let Err(e) = shell::run_bash_output(&script) {
            return Err(format!("Failed to install Git: {}", e));
        }
    }
    
    Ok(InstallResult {
        success: true,
        message: "Git installed successfully from local bundle!".to_string(),
        error: None,
    })
}

/// Install OpenClaw from local bundle
#[command]
pub async fn install_openclaw() -> Result<InstallResult, String> {
    info!("[Local Bundle Install] Installing OpenClaw from local bundle...");
    
    let bundles_dir = get_runtime_bundles_dir()?;
    
    let openclaw_dir = format!("{}/common/openclaw", bundles_dir);
    let openclaw_file = "openclaw-2026.3.12.tgz";
    let openclaw_file_path = format!("{}/{}", openclaw_dir, openclaw_file);
    
    info!("[Local Bundle Install] OpenClaw bundle: {}", openclaw_file_path);
    
    if !Path::new(&openclaw_file_path).exists() {
        return Err(format!("OpenClaw bundle not found: {}", openclaw_file_path));
    }
    
    let extract_dir = format!("{}/temp/openclaw", bundles_dir);
    if let Err(e) = fs::create_dir_all(&extract_dir) {
        return Err(format!("Failed to create extraction directory: {}", e));
    }
    
    info!("[Local Bundle Install] Extracting OpenClaw bundle...");
    let extract_script = format!(r#"
tar -xzf '{}' -C '{}'
"#, openclaw_file_path, extract_dir);
    if let Err(e) = shell::run_bash_output(&extract_script) {
        return Err(format!("Failed to extract OpenClaw bundle: {}", e));
    }
    
    info!("[Local Bundle Install] Installing OpenClaw...");
    let install_script = format!(r#"
cd '{}'
npm install '{}' --save-exact
openclaw gateway start
openclaw gateway status
"#, extract_dir, openclaw_file_path);
    if let Err(e) = shell::run_bash_output(&install_script) {
        return Err(format!("Failed to install OpenClaw: {}", e));
    }
    
    if let Err(e) = fs::remove_dir_all(&extract_dir) {
        warn!("[Local Bundle Install] Failed to clean up temporary directory: {}", e);
    }
    
    Ok(InstallResult {
        success: true,
        message: "OpenClaw installed successfully from local bundle!".to_string(),
        error: None,
    })
}

/// Install all components from local bundles
#[command]
pub async fn install_all_from_local() -> Result<InstallResult, String> {
    info!("[Local Bundle Install] Starting installation of all components from local bundles...");
    
    // Install Node.js
    let node_result = install_nodejs().await?;
    if !node_result.success {
        return Ok(node_result);
    }
    
    // Install Git
    let git_result = install_git().await?;
    if !git_result.success {
        return Ok(git_result);
    }
    
    // Install OpenClaw
    let openclaw_result = install_openclaw().await?;
    if !openclaw_result.success {
        return Ok(openclaw_result);
    }
    
    Ok(InstallResult {
        success: true,
        message: "All components installed successfully from local bundles!".to_string(),
        error: None,
    })
}

/// Initialize OpenClaw configuration
#[command]
pub async fn init_openclaw_config() -> Result<InstallResult, String> {
    info!("[Init Config] Starting OpenClaw configuration initialization...");

    let config_dir = platform::get_config_dir();
    info!("[Init Config] Config directory: {}", config_dir);

    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        error!("[Init Config] Failed to create config directory: {}", e);
        return Ok(InstallResult {
            success: false,
            message: "Failed to create config directory".to_string(),
            error: Some(e.to_string()),
        });
    }

    let subdirs = ["agents/main/sessions", "agents/main/agent", "credentials"];
    for subdir in subdirs {
        let path = format!("{}/{}", config_dir, subdir);
        info!("[Init Config] Creating subdirectory: {}", subdir);
        if let Err(e) = std::fs::create_dir_all(&path) {
            error!("[Init Config] Failed to create directory: {} - {}", subdir, e);
            return Ok(InstallResult {
                success: false,
                message: format!("Failed to create directory: {}", subdir),
                error: Some(e.to_string()),
            });
        }
    }

    #[cfg(unix)]
    {
        info!("[Init Config] Setting directory permissions to 700...");
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&config_dir) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o700);
            if let Err(e) = std::fs::set_permissions(&config_dir, perms) {
                warn!("[Init Config] Failed to set permissions: {}", e);
            } else {
                info!("[Init Config] Permissions set successfully");
            }
        }
    }

    info!("[Init Config] Executing: openclaw config set gateway.mode local");
    let result = shell::run_openclaw(&["config", "set", "gateway.mode", "local"]);

    info!("[Init Config] Executing: openclaw config set gateway.controlUi.allowInsecureAuth true");
    let _ = shell::run_openclaw(&["config", "set", "gateway.controlUi.allowInsecureAuth", "true"]);

    match result {
        Ok(output) => {
            info!("[Init Config] Configuration initialized successfully");
            debug!("[Init Config] Command output: {}", log_sanitizer::sanitize(&output));
            Ok(InstallResult {
                success: true,
                message: "Configuration initialized successfully!".to_string(),
                error: None,
            })
        },
        Err(e) => {
            error!("[Init Config] Configuration initialization failed: {}", e);
            Ok(InstallResult {
                success: false,
                message: "Configuration initialization failed".to_string(),
                error: Some(e),
            })
        },
    }
}

/// Uninstall OpenClaw
#[command]
pub async fn uninstall_openclaw() -> Result<InstallResult, String> {
    info!("[Uninstall OpenClaw] Starting OpenClaw uninstallation...");
    let os = platform::get_os();
    info!("[Uninstall OpenClaw] Detected operating system: {}", os);

    let _ = shell::run_openclaw(&["gateway", "stop"]);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let result = if platform::is_windows() {
        match shell::run_cmd_output("npm uninstall -g openclaw") {
            Ok(output) => {
                info!("[Uninstall OpenClaw] npm output: {}", output);
                std::thread::sleep(std::time::Duration::from_millis(500));
                if get_openclaw_version().is_none() {
                    Ok(InstallResult {
                        success: true,
                        message: "OpenClaw has been successfully uninstalled!".to_string(),
                        error: None,
                    })
                } else {
                    Ok(InstallResult {
                        success: false,
                        message: "Uninstall command executed but OpenClaw still exists".to_string(),
                        error: Some(output),
                    })
                }
            }
            Err(e) => {
                warn!("[Uninstall OpenClaw] npm uninstall failed: {}", e);
                Ok(InstallResult {
                    success: false,
                    message: "OpenClaw uninstallation failed".to_string(),
                    error: Some(e),
                })
            }
        }
    } else {
        let script = r#"
echo "Uninstalling OpenClaw..."
npm uninstall -g openclaw
if command -v openclaw &> /dev/null; then
    echo "Warning: openclaw command still exists"
    exit 1
else
    echo "OpenClaw has been successfully uninstalled"
    exit 0
fi
"#;
        match shell::run_bash_output(script) {
            Ok(output) => Ok(InstallResult {
                success: true,
                message: format!("OpenClaw has been successfully uninstalled! {}", output),
                error: None,
            }),
            Err(e) => Ok(InstallResult {
                success: false,
                message: "OpenClaw uninstallation failed".to_string(),
                error: Some(e),
            }),
        }
    };

    if let Some(home) = dirs::home_dir() {
        let openclaw_dir = home.join(".openclaw");
        if openclaw_dir.exists() {
            info!("[Uninstall OpenClaw] Deleting .openclaw directory: {:?}", openclaw_dir);
            match std::fs::remove_dir_all(&openclaw_dir) {
                Ok(_) => info!("[Uninstall OpenClaw] Successfully deleted .openclaw directory"),
                Err(e) => warn!("[Uninstall OpenClaw] Failed to delete .openclaw directory: {}", e),
            }
        }
    }

    result
}
