fn main() {
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
    
    // 监听资源文件变化，自动重新编译
    println!("cargo:rerun-if-changed=assets/logo.ico");
    println!("cargo:rerun-if-changed=assets/logo-black.png");
    println!("cargo:rerun-if-changed=assets/logo-white.png");
}
