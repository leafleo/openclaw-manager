use std::process::Command;
use std::env;

fn main() {
    println!("Testing run_openclaw equivalent...");
    
    // Get openclaw path
    let openclaw_path = Command::new("which")
        .arg("openclaw")
        .output()
        .expect("Failed to find openclaw")
        .stdout;
    let openclaw_path = String::from_utf8_lossy(&openclaw_path).trim().to_string();
    println!("OpenClaw path: {}", openclaw_path);
    
    // Get package directory
    let package_dir = if let Ok(canonical) = std::fs::canonicalize(&openclaw_path) {
        let path_str = canonical.to_string_lossy().to_string();
        if let Some(pos) = path_str.find("/runtime-bundles/common/openclaw/package") {
            path_str[..pos + "/runtime-bundles/common/openclaw/package".len()].to_string()
        } else {
            ".".to_string()
        }
    } else {
        ".".to_string()
    };
    println!("Package directory: {}", package_dir);
    
    // Test health command
    println!("\nExecuting health command...");
    let mut cmd = Command::new(&openclaw_path);
    cmd.args(["gateway", "health", "--timeout", "3000"])
        .current_dir(&package_dir);
    
    let output = cmd.output().expect("Failed to execute command");
    println!("Exit code: {:?}", output.status.code());
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout).trim());
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr).trim());
    println!("Success: {}", output.status.success());
}