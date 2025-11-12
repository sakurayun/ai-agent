use image::ImageReader;
use std::io::Cursor;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIconBuilder,
};

pub fn create_tray_icon() -> Option<tray_icon::TrayIcon> {
    // macOS 使用黑色 logo（会自动适应系统主题），Windows 使用黑色 logo
    // macOS 系统托盘会自动处理图标颜色以适应浅色/深色模式
    let icon_bytes = include_bytes!("../assets/logo-black.png");

    match load_icon_from_bytes(icon_bytes) {
        Ok(icon) => {
            // 创建托盘菜单
            let menu = Menu::new();
            let show_item = MenuItem::new("Show", true, None);
            let quit_item = MenuItem::new("Quit", true, None);

            if let Err(e) = menu.append(&show_item) {
                eprintln!("Failed to add show menu item: {}", e);
                return None;
            }

            if let Err(e) = menu.append(&quit_item) {
                eprintln!("Failed to add quit menu item: {}", e);
                return None;
            }

            // 创建托盘图标
            let builder = TrayIconBuilder::new()
                .with_icon(icon)
                .with_tooltip("AI Agent")
                .with_menu(Box::new(menu));

            // macOS 特定配置：使用模板模式，这样图标会自动适应系统主题
            #[cfg(target_os = "macos")]
            let builder = builder.with_icon_as_template(true);
            
            #[cfg(not(target_os = "macos"))]
            let builder = builder;

            match builder.build() {
                Ok(tray) => {
                    println!("System tray icon created successfully");
                    Some(tray)
                }
                Err(e) => {
                    eprintln!("Failed to create tray icon: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load tray icon: {}", e);
            None
        }
    }
}

fn load_icon_from_bytes(bytes: &[u8]) -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // macOS 系统托盘图标通常使用 16x16 或 22x22 (Retina: 32x32 或 44x44)
    // Windows 系统托盘图标通常需要 16x16 或 32x32
    #[cfg(target_os = "macos")]
    let target_size = 22u32; // macOS 标准尺寸

    #[cfg(target_os = "windows")]
    let target_size = 32u32; // Windows 标准尺寸

    let resized = if width > target_size || height > target_size {
        image::imageops::resize(
            &rgba,
            target_size,
            target_size,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        rgba
    };

    let (width, height) = resized.dimensions();
    let icon = tray_icon::Icon::from_rgba(resized.into_raw(), width, height)?;

    Ok(icon)
}
