use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem}};
use image::ImageReader;
use std::io::Cursor;

pub fn create_tray_icon() -> Option<tray_icon::TrayIcon> {
    // 加载黑色logo作为托盘图标
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
            match TrayIconBuilder::new()
                .with_icon(icon)
                .with_tooltip("AI Agent")
                .with_menu(Box::new(menu))
                .build()
            {
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
    
    // Windows系统托盘图标通常需要16x16或32x32
    // 如果图片太大，需要缩放
    let resized = if width > 32 || height > 32 {
        image::imageops::resize(&rgba, 32, 32, image::imageops::FilterType::Lanczos3)
    } else {
        rgba
    };
    
    let (width, height) = resized.dimensions();
    let icon = tray_icon::Icon::from_rgba(resized.into_raw(), width, height)?;
    
    Ok(icon)
}
