use std::fs;
use std::path::Path;

fn main() {
    println!("Testing openclaw paths...");
    
    // Test 1: Check if openclaw is in PATH
    let output = std::process::Command::new("which")
        .arg("openclaw")
        .output()
        .expect("Failed to execute which command");
    
    println!("which openclaw:");
    println!("Exit code: {:?}", output.status.code());
    println!("Path: {}", String::from_utf8_lossy(&output.stdout).trim());
    
    // Test 2: Check the canonical path
    let openclaw_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !openclaw_path.is_empty() {
        println!("\nCanonical path:");
        if let Ok(canonical) = fs::canonicalize(&openclaw_path) {
            println!("Canonical: {}", canonical.display());
            
            // Test 3: Check if package directory exists
            let path_str = canonical.to_string_lossy().to_string();
            if let Some(pos) = path_str.find("/lib/node_modules/openclaw") {
                let package_dir = &path_str[..pos + "/lib/node_modules/openclaw".len()];
                println!("\nPackage directory: {}", package_dir);
                println!("Exists: {}", Path::new(package_dir).exists());
            }
        }
    }
    
    // Test 4: Check nvm default version
    println!("\nNVM default version:");
    if let Ok(version) = fs::read_to_string("~/.nvm/alias/default") {
        let version = version.trim();
        println!("NVM default: {}", version);
        
        // Test 5: Check nvm path
        let home = std::env::var("HOME").unwrap_or("~".to_string());
        let version_str = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };
        let nvm_path = format!("{}/.nvm/versions/node/{}/lib/node_modules/openclaw", home, version_str);
        println!("NVM package path: {}", nvm_path);
        println!("Exists: {}", Path::new(&nvm_path).exists());
    }
}