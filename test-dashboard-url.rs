use std::process::Command;

fn main() {
    println!("Testing get_dashboard_url equivalent...");
    
    // Get openclaw config
    let config_output = Command::new("openclaw")
        .args(["config", "get", "gateway.auth.token"])
        .output()
        .expect("Failed to get config");
    
    let token = String::from_utf8_lossy(&config_output.stdout).trim().to_string();
    println!("Token from config: {}", if token.is_empty() { "(empty)" } else { "(exists)" });
    
    if token.is_empty() {
        println!("Token is empty, checking if we need to generate one...");
        
        // Check if config exists
        let config_json = Command::new("openclaw")
            .args(["config", "get", "--json"])
            .output()
            .expect("Failed to get config json");
        
        println!("Config output: {}", String::from_utf8_lossy(&config_json.stdout));
    } else {
        let url = format!("http://localhost:18789?token={}", token);
        println!("Dashboard URL: {}", url);
    }
}