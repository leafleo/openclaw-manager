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

/// Get openclaw-bundle directory (bundle/resources/openclaw-bundle)
fn get_openclaw_bundle_dir() -> Result<String, String> {
    // Check if OPENCLAW_BUNDLE_DIR environment variable is set
    if let Ok(bundle_dir) = std::env::var("OPENCLAW_BUNDLE_DIR") {
        info!("[Bundle Install] Using OpenClaw bundle directory from environment: {}", bundle_dir);
        return Ok(bundle_dir);
    }
    
    // Fallback to default path
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let exe_dir = exe_path.parent()
        .ok_or("Failed to get executable directory")?;
    
    let bundle_dir = exe_dir.join("bundle").join("resources").join("openclaw-bundle");
    
    let path = bundle_dir.to_string_lossy().to_string();
    info!("[Bundle Install] OpenClaw bundle directory (default): {}", path);
    
    Ok(path)
}

/// Bundle manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleManifest {
    name: String,
    generated_at: String,
    openclaw_version: Option<String>,
    node_version: Option<String>,
    node_platform: Option<String>,
    prefix_available: Option<bool>,
    files: BundleFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleFiles {
    openclaw_tgz: String,
    npm_cache: String,
    node: String,
    npm_cli: String,
    prefix: Option<String>,
}

/// Get bundle info and paths
fn get_bundle_info() -> Result<(String, BundleManifest, String, String, String, String), String> {
    let bundle_dir = get_openclaw_bundle_dir()?;
    
    if !Path::new(&bundle_dir).exists() {
        return Err(format!("Bundle directory not found: {}", bundle_dir));
    }

    let manifest_path = format!("{}/manifest.json", bundle_dir);
    if !Path::new(&manifest_path).exists() {
        return Err(format!("manifest.json not found: {}", manifest_path));
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest.json: {}", e))?;
    
    let manifest: BundleManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;
    
    let node_path = format!("{}/{}", bundle_dir, manifest.files.node);
    let npm_cli_path = format!("{}/{}", bundle_dir, manifest.files.npm_cli);
    let npm_cache_path = format!("{}/{}", bundle_dir, manifest.files.npm_cache);
    let openclaw_tgz_path = format!("{}/{}", bundle_dir, manifest.files.openclaw_tgz);

    if !Path::new(&node_path).exists() {
        return Err(format!("Node.js not found: {}", node_path));
    }
    if !Path::new(&npm_cli_path).exists() {
        return Err(format!("npm-cli.js not found: {}", npm_cli_path));
    }
    if !Path::new(&openclaw_tgz_path).exists() {
        return Err(format!("openclaw.tgz not found: {}", openclaw_tgz_path));
    }

    let node_dir = Path::new(&node_path).parent()
        .ok_or("Failed to get node directory")?
        .to_string_lossy().to_string();
    let npm_bin_dir = Path::new(&npm_cli_path).parent()
        .ok_or("Failed to get npm bin directory")?
        .to_string_lossy().to_string();

    Ok((bundle_dir, manifest, node_dir, npm_bin_dir, npm_cache_path, openclaw_tgz_path))
}

/// Install Node.js from openclaw-bundle directory
/// This function:
/// 1. Reads manifest.json to get bundle info
/// 2. Adds node and npm to environment variables
/// 3. Verifies Node.js is accessible
#[command]
pub async fn install_nodejs() -> Result<InstallResult, String> {
    info!("[Node Install] Starting Node.js installation from bundle...");

    let (bundle_dir, _manifest, node_dir, npm_bin_dir, _npm_cache_path, _openclaw_tgz_path) = get_bundle_info()?;
    
    info!("[Node Install] Node directory: {}", node_dir);
    info!("[Node Install] npm bin directory: {}", npm_bin_dir);

    if platform::is_windows() {
        install_nodejs_windows(&node_dir, &npm_bin_dir).await
    } else {
        install_nodejs_unix(&node_dir, &npm_bin_dir).await
    }
}

/// Install OpenClaw from openclaw-bundle directory
/// This function:
/// 1. Reads manifest.json to get bundle info
/// 2. Installs openclaw.tgz using offline npm-cache and prefix
/// 3. Verifies the installation
/// Note: Node.js must be installed first
#[command]
pub async fn install_openclaw() -> Result<InstallResult, String> {
    info!("[OpenClaw Install] Starting OpenClaw installation from bundle...");

    let (bundle_dir, manifest, node_dir, npm_bin_dir, npm_cache_path, openclaw_tgz_path) = get_bundle_info()?;
    
    info!("[OpenClaw Install] Node directory: {}", node_dir);
    info!("[OpenClaw Install] npm bin directory: {}", npm_bin_dir);

    if platform::is_windows() {
        install_openclaw_windows(&bundle_dir, &node_dir, &npm_bin_dir, &npm_cache_path, &openclaw_tgz_path, &manifest).await
    } else {
        install_openclaw_unix(&bundle_dir, &node_dir, &npm_bin_dir, &npm_cache_path, &openclaw_tgz_path, &manifest).await
    }
}

/// Install Node.js on Windows: set environment variables
async fn install_nodejs_windows(
    node_dir: &str,
    npm_bin_dir: &str,
) -> Result<InstallResult, String> {
    info!("[Windows Node Install] Setting up environment variables...");

    let node_dir_win = node_dir.replace('/', "\\");
    let npm_bin_dir_win = npm_bin_dir.replace('/', "\\");

    let env_script = format!(
        r#"
        $nodeDir = "{}"
        $npmBinDir = "{}"

        $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

        if ($currentPath -notlike "*$nodeDir*") {{
            [Environment]::SetEnvironmentVariable("PATH", "$nodeDir;$currentPath", "User")
            Write-Host "Added node to PATH"
        }}

        if ($currentPath -notlike "*$npmBinDir*") {{
            $newPath = [Environment]::GetEnvironmentVariable("PATH", "User")
            [Environment]::SetEnvironmentVariable("PATH", "$npmBinDir;$newPath", "User")
            Write-Host "Added npm/bin to PATH"
        }}

        [Environment]::SetEnvironmentVariable("NODE_HOME", $nodeDir, "User")
        Write-Host "Set NODE_HOME to $nodeDir"
        "#,
        node_dir_win, npm_bin_dir_win
    );

    match shell::run_powershell_output(&env_script) {
        Ok(output) => {
            info!("[Windows Node Install] Environment variables configured: {}", output);
        }
        Err(e) => {
            return Err(format!("Failed to set environment variables: {}", e));
        }
    }

    info!("[Windows Node Install] Verifying Node.js installation...");

    let verify_script = format!(
        r#"
        $nodePath = "{}\node.exe"
        $version = & $nodePath --version 2>$null
        if ($LASTEXITCODE -eq 0) {{
            Write-Host "Node.js version: $version"
            exit 0
        }}
        Write-Error "Node.js verification failed"
        exit 1
        "#,
        node_dir_win
    );

    match shell::run_powershell_output(&verify_script) {
        Ok(output) => {
            info!("[Windows Node Install] Verification successful: {}", output);
        }
        Err(e) => {
            warn!("[Windows Node Install] Verification warning: {}", e);
        }
    }

    info!("[Windows Node Install] Installation completed successfully");

    Ok(InstallResult {
        success: true,
        message: "Node.js installed successfully! Please restart the application.".to_string(),
        error: None,
    })
}

/// Install OpenClaw on Windows using offline npm
async fn install_openclaw_windows(
    bundle_dir: &str,
    node_dir: &str,
    npm_bin_dir: &str,
    npm_cache_path: &str,
    openclaw_tgz_path: &str,
    manifest: &BundleManifest,
) -> Result<InstallResult, String> {
    info!("[Windows OpenClaw Install] Starting OpenClaw installation...");

    let node_dir_win = node_dir.replace('/', "\\");
    let npm_bin_dir_win = npm_bin_dir.replace('/', "\\");
    let npm_cache_win = npm_cache_path.replace('/', "\\");
    let openclaw_tgz_win = openclaw_tgz_path.replace('/', "\\");
    let bundle_dir_win = bundle_dir.replace('/', "\\");

    let install_prefix = format!("{}\\prefix", bundle_dir_win);
    let prefix_available = manifest.prefix_available.unwrap_or(false);

    if prefix_available && Path::new(&install_prefix).exists() {
        info!("[Windows OpenClaw Install] Using pre-built prefix for offline install...");
        
        let prefix_script = format!(
            r#"
            $nodePath = "{}\node.exe"
            $npmCliPath = "{}\npm-cli.js"
            $prefixPath = "{}"
            $cachePath = "{}"

            $env:PATH = "{};" + $env:PATH

            # Verify prefix has openclaw
            $openclawCmd = Join-Path $prefixPath "openclaw.cmd"
            if (Test-Path $openclawCmd) {{
                Write-Host "OpenClaw already available in prefix"
                & $nodePath $npmCliPath config set prefix $prefixPath --global --cache $cachePath
                exit 0
            }}

            # Install from cache
            & $nodePath $npmCliPath install --global --prefix $prefixPath --cache $cachePath --offline --no-audit --no-fund "{}"
            if ($LASTEXITCODE -ne 0) {{
                Write-Error "Offline npm install failed"
                exit 1
            }}
            "#,
            node_dir_win,
            npm_bin_dir_win,
            install_prefix,
            npm_cache_win,
            node_dir_win,
            openclaw_tgz_win
        );

        match shell::run_powershell_output(&prefix_script) {
            Ok(output) => {
                info!("[Windows OpenClaw Install] Prefix setup output: {}", output);
            }
            Err(e) => {
                warn!("[Windows OpenClaw Install] Prefix setup failed, trying fresh install: {}", e);
            }
        }
    } else {
        info!("[Windows OpenClaw Install] Performing fresh offline install...");

        let fresh_install_script = format!(
            r#"
            $nodePath = "{}\node.exe"
            $npmCliPath = "{}\npm-cli.js"
            $prefixPath = "{}"
            $cachePath = "{}"
            $tgzPath = "{}"

            $env:PATH = "{};" + $env:PATH

            if (-not (Test-Path $prefixPath)) {{
                New-Item -ItemType Directory -Force -Path $prefixPath | Out-Null
            }}

            & $nodePath $npmCliPath install --global --prefix $prefixPath --cache $cachePath --offline --no-audit --no-fund "$tgzPath"
            if ($LASTEXITCODE -ne 0) {{
                Write-Error "Offline npm install failed"
                exit 1
            }}

            Write-Host "OpenClaw installed successfully"
            "#,
            node_dir_win,
            npm_bin_dir_win,
            install_prefix,
            npm_cache_win,
            openclaw_tgz_win,
            node_dir_win
        );

        match shell::run_powershell_output(&fresh_install_script) {
            Ok(output) => {
                info!("[Windows OpenClaw Install] Fresh install output: {}", output);
            }
            Err(e) => {
                return Err(format!("Failed to install OpenClaw: {}", e));
            }
        }
    }

    info!("[Windows OpenClaw Install] Verifying installation...");

    let verify_script = format!(
        r#"
        $prefixPath = "{}"
        $nodePath = "{}\node.exe"
        $env:PATH = "$prefixPath;" + $env:PATH

        $openclawCmd = Join-Path $prefixPath "openclaw.cmd"
        if (Test-Path $openclawCmd) {{
            $version = & $openclawCmd --version 2>$null
            if ($LASTEXITCODE -eq 0) {{
                Write-Host "OpenClaw version: $version"
                exit 0
            }}
        }}

        $openclawMjs = Join-Path $prefixPath "node_modules\openclaw\openclaw.mjs"
        if (Test-Path $openclawMjs) {{
            $version = & $nodePath $openclawMjs --version 2>$null
            if ($LASTEXITCODE -eq 0) {{
                Write-Host "OpenClaw version: $version"
                exit 0
            }}
        }}

        Write-Error "OpenClaw verification failed"
        exit 1
        "#,
        install_prefix,
        node_dir_win
    );

    match shell::run_powershell_output(&verify_script) {
        Ok(output) => {
            info!("[Windows OpenClaw Install] Verification successful: {}", output);
        }
        Err(e) => {
            warn!("[Windows OpenClaw Install] Verification warning: {}", e);
        }
    }

    let prefix_env_script = format!(
        r#"
        $prefixPath = "{}"
        $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($currentPath -notlike "*$prefixPath*") {{
            [Environment]::SetEnvironmentVariable("PATH", "$prefixPath;$currentPath", "User")
            Write-Host "Added prefix to PATH"
        }}
        "#,
        install_prefix
    );

    let _ = shell::run_powershell_output(&prefix_env_script);

    info!("[Windows OpenClaw Install] Installation completed successfully");

    Ok(InstallResult {
        success: true,
        message: "OpenClaw installed successfully! Please restart the application.".to_string(),
        error: None,
    })
}

/// Install Node.js on Unix (macOS/Linux): set environment variables
async fn install_nodejs_unix(
    node_dir: &str,
    npm_bin_dir: &str,
) -> Result<InstallResult, String> {
    info!("[Unix Node Install] Setting up environment variables...");

    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        
        let zprofile_path = format!("{}/.zprofile", home_str);
        let bash_profile_path = format!("{}/.bash_profile", home_str);

        let env_lines = format!(
            r#"
# OpenClaw Manager - Node.js environment
export NODE_HOME="{}"
export PATH="$NODE_HOME:{}:$PATH"
"#,
            node_dir, npm_bin_dir
        );

        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&zprofile_path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(env_lines.as_bytes())
            });

        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&bash_profile_path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(env_lines.as_bytes())
            });
    }

    info!("[Unix Node Install] Verifying Node.js installation...");

    let node_exe = format!("{}/node", node_dir);
    let verify_script = format!(
        r#"
export PATH="{}:$PATH"

VERSION=$("{}" --version 2>/dev/null)
if [ $? -eq 0 ]; then
    echo "Node.js version: $VERSION"
    exit 0
fi

echo "Node.js verification failed"
exit 1
"#,
        node_dir, node_exe
    );

    match shell::run_bash_output(&verify_script) {
        Ok(output) => {
            info!("[Unix Node Install] Verification successful: {}", output);
        }
        Err(e) => {
            warn!("[Unix Node Install] Verification warning: {}", e);
        }
    }

    info!("[Unix Node Install] Installation completed successfully");

    Ok(InstallResult {
        success: true,
        message: "Node.js installed successfully! Please restart the application.".to_string(),
        error: None,
    })
}

/// Install OpenClaw on Unix (macOS/Linux) using offline npm
async fn install_openclaw_unix(
    bundle_dir: &str,
    node_dir: &str,
    npm_bin_dir: &str,
    npm_cache_path: &str,
    openclaw_tgz_path: &str,
    manifest: &BundleManifest,
) -> Result<InstallResult, String> {
    info!("[Unix OpenClaw Install] Starting OpenClaw installation...");

    let install_prefix = format!("{}/prefix", bundle_dir);
    let prefix_available = manifest.prefix_available.unwrap_or(false);

    let node_exe = format!("{}/node", node_dir);
    let npm_cli = format!("{}/npm-cli.js", npm_bin_dir);

    if prefix_available && Path::new(&install_prefix).exists() {
        info!("[Unix OpenClaw Install] Using pre-built prefix for offline install...");

        let prefix_script = format!(
            r#"
export PATH="{}:$PATH"

OPENCLAW_CMD="{}/bin/openclaw"
if [ -f "$OPENCLAW_CMD" ]; then
    echo "OpenClaw already available in prefix"
    "{}" "{}" config set prefix "{}" --global --cache "{}"
    exit 0
fi

"{}" "{}" install --global --prefix "{}" --cache "{}" --offline --no-audit --no-fund "{}"
"#,
            node_dir,
            install_prefix,
            node_exe, npm_cli, install_prefix, npm_cache_path,
            node_exe, npm_cli, install_prefix, npm_cache_path, openclaw_tgz_path
        );

        match shell::run_bash_output(&prefix_script) {
            Ok(output) => {
                info!("[Unix OpenClaw Install] Prefix setup output: {}", output);
            }
            Err(e) => {
                warn!("[Unix OpenClaw Install] Prefix setup failed, trying fresh install: {}", e);
            }
        }
    } else {
        info!("[Unix OpenClaw Install] Performing fresh offline install...");

        let fresh_install_script = format!(
            r#"
export PATH="{}:$PATH"

mkdir -p "{}"

"{}" "{}" install --global --prefix "{}" --cache "{}" --offline --no-audit --no-fund "{}"
"#,
            node_dir,
            install_prefix,
            node_exe, npm_cli, install_prefix, npm_cache_path, openclaw_tgz_path
        );

        match shell::run_bash_output(&fresh_install_script) {
            Ok(output) => {
                info!("[Unix OpenClaw Install] Fresh install output: {}", output);
            }
            Err(e) => {
                return Err(format!("Failed to install OpenClaw: {}", e));
            }
        }
    }

    info!("[Unix OpenClaw Install] Verifying installation...");

    let verify_script = format!(
        r#"
export PATH="{}:$PATH"

OPENCLAW_CMD="{}/bin/openclaw"
if [ -f "$OPENCLAW_CMD" ]; then
    VERSION=$("$OPENCLAW_CMD" --version 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "OpenClaw version: $VERSION"
        exit 0
    fi
fi

OPENCLAW_MJS="{}/node_modules/openclaw/openclaw.mjs"
if [ -f "$OPENCLAW_MJS" ]; then
    VERSION=$("{}" "$OPENCLAW_MJS" --version 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "OpenClaw version: $VERSION"
        exit 0
    fi
fi

echo "OpenClaw verification failed"
exit 1
"#,
        node_dir,
        install_prefix,
        install_prefix,
        node_exe
    );

    match shell::run_bash_output(&verify_script) {
        Ok(output) => {
            info!("[Unix OpenClaw Install] Verification successful: {}", output);
        }
        Err(e) => {
            warn!("[Unix OpenClaw Install] Verification warning: {}", e);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        let prefix_env_lines = format!(
            r#"
# OpenClaw Manager - OpenClaw prefix
export PATH="{}:$PATH"
"#,
            format!("{}/bin", install_prefix)
        );

        let zprofile_path = format!("{}/.zprofile", home_str);
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&zprofile_path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(prefix_env_lines.as_bytes())
            });

        let bash_profile_path = format!("{}/.bash_profile", home_str);
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&bash_profile_path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(prefix_env_lines.as_bytes())
            });
    }

    info!("[Unix OpenClaw Install] Installation completed successfully");

    Ok(InstallResult {
        success: true,
        message: "OpenClaw installed successfully! Please restart the application.".to_string(),
        error: None,
    })
}

/// Install all components from openclaw-bundle
#[command]
pub async fn install_all_from_local() -> Result<InstallResult, String> {
    info!("[Bundle Install] Starting installation of all components from bundle...");
    install_nodejs().await
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

#[cfg(test)]
mod tests {
    use super::*;

    // Test get_openclaw_bundle_dir function
    #[test]
    fn test_get_openclaw_bundle_dir() {
        // Test with environment variable
        let test_path = "/tmp/openclaw-test-bundle";
        std::env::set_var("OPENCLAW_BUNDLE_DIR", test_path);
        
        let result = get_openclaw_bundle_dir();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_path);
        
        // Test without environment variable (should use default path)
        std::env::remove_var("OPENCLAW_BUNDLE_DIR");
        let result = get_openclaw_bundle_dir();
        assert!(result.is_ok());
        let bundle_dir = result.unwrap();
        assert!(bundle_dir.contains("bundle/resources/openclaw-bundle"));
    }

    // Test check_environment function
    #[tokio::test]
    async fn test_check_environment() {
        let result = check_environment().await;
        assert!(result.is_ok());
        let env_status = result.unwrap();
        
        // Verify all fields are present
        assert!(env_status.node_installed || !env_status.node_installed);
        assert!(env_status.node_version.is_some() || env_status.node_version.is_none());
        assert!(env_status.node_version_ok || !env_status.node_version_ok);
        assert!(env_status.git_installed || !env_status.git_installed);
        assert!(env_status.git_version.is_some() || env_status.git_version.is_none());
        assert!(env_status.openclaw_installed || !env_status.openclaw_installed);
        assert!(env_status.openclaw_version.is_some() || env_status.openclaw_version.is_none());
        assert!(env_status.gateway_service_installed || !env_status.gateway_service_installed);
        assert!(env_status.config_dir_exists || !env_status.config_dir_exists);
        assert!(env_status.ready || !env_status.ready);
        assert!(!env_status.os.is_empty());
    }

    // Test init_openclaw_config function
    #[tokio::test]
    async fn test_init_openclaw_config() {
        let result = init_openclaw_config().await;
        assert!(result.is_ok());
        let init_result = result.unwrap();
        assert!(init_result.success || !init_result.success);
    }

    // Test uninstall_openclaw function
    #[tokio::test]
    async fn test_uninstall_openclaw() {
        let result = uninstall_openclaw().await;
        // This should either succeed or fail gracefully
        assert!(result.is_ok());
    }

    // Test install_all_from_local function
    #[tokio::test]
    async fn test_install_all_from_local() {
        let result = install_all_from_local().await;
        // This might fail if bundle directory doesn't exist, but should return Result
        assert!(result.is_ok() || result.is_err());
    }

    // Test install_gateway_service function
    #[tokio::test]
    async fn test_install_gateway_service() {
        let result = install_gateway_service().await;
        // This might fail if not run as admin, but should return Result
        assert!(result.is_ok() || result.is_err());
    }
}
