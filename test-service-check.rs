use std::process::Command;

fn main() {
    println!("Testing service status check...");
    
    // Test 1: Check port listening
    println!("\n1. Checking port 18789...");
    let lsof_output = Command::new("lsof")
        .args(["-ti", ":18789"])
        .output();
    
    match lsof_output {
        Ok(output) => {
            if output.status.success() {
                let pids = String::from_utf8_lossy(&output.stdout);
                println!("Port 18789 is in use by PIDs: {}", pids.trim());
            } else {
                println!("Port 18789 is not in use");
            }
        }
        Err(e) => println!("Failed to check port: {}", e),
    }
    
    // Test 2: Check gateway health
    println!("\n2. Checking gateway health...");
    let health_output = Command::new("openclaw")
        .args(["gateway", "health", "--timeout", "3000"])
        .output();
    
    match health_output {
        Ok(output) => {
            println!("Exit code: {:?}", output.status.code());
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
            println!("Success: {}", output.status.success());
        }
        Err(e) => println!("Failed to check health: {}", e),
    }
}