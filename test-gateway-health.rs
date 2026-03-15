use std::process::Command;

fn main() {
    println!("Testing openclaw gateway health...");
    
    let output = Command::new("openclaw")
        .args(["gateway", "health", "--timeout", "3000"])
        .output()
        .expect("Failed to execute command");
    
    println!("Exit code: {:?}", output.status.code());
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    if output.status.success() {
        println!("✅ Health check passed!");
    } else {
        println!("❌ Health check failed!");
    }
}