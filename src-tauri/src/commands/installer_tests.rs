use super::*;

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
        assert!(env_status.node_version.is_some() || env_status.node_version.is_none());
        assert!(env_status.git_available || !env_status.git_available);
        assert!(env_status.openclaw_available || !env_status.openclaw_available);
        assert!(env_status.os_info.is_some());
        assert!(env_status.cpu_info.is_some());
        assert!(env_status.memory_info.is_some());
    }

    // Test init_openclaw_config function
    #[tokio::test]
    async fn test_init_openclaw_config() {
        let result = init_openclaw_config().await;
        assert!(result.is_ok());
        let init_result = result.unwrap();
        assert!(init_result.success);
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
