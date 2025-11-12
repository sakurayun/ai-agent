fn main() {
    // Windows 平台配置
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        
        // 设置Windows应用图标（任务栏、Alt+Tab等）
        res.set_icon("assets/logo.ico");
        res.set("FileDescription", "AI Agent");
        res.set("ProductName", "AI Agent");
        res.set("CompanyName", "AI Agent");
        res.set("FileVersion", "0.1.0");
        res.set("ProductVersion", "0.1.0");
        
        if let Err(e) = res.compile() {
            eprintln!("Error: Failed to set Windows icon: {}", e);
            std::process::exit(1);
        }
    }
    
    // macOS 平台配置
    #[cfg(target_os = "macos")]
    {
        // macOS 的图标通过 cargo-bundle 或应用程序包配置
        // 确保 .icns 文件存在
        let icns_path = std::path::Path::new("assets/logo.icns");
        if !icns_path.exists() {
            eprintln!("Warning: macOS icon file not found at assets/logo.icns");
            eprintln!("Run ./generate_icns.sh to generate the icon file");
        } else {
            println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");
            println!("Found macOS icon file: assets/logo.icns");
        }
    }
    
    // 监听资源文件变化，自动重新编译
    println!("cargo:rerun-if-changed=assets/logo.ico");
    println!("cargo:rerun-if-changed=assets/logo.icns");
    println!("cargo:rerun-if-changed=assets/logo-black.png");
    println!("cargo:rerun-if-changed=assets/logo-white.png");
}
