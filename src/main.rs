use gpui::*;
use gpui_component::*;

mod api;
mod app;
mod assets;
mod components;
mod state;
mod utils;
mod views;

use app::App;
use assets::Assets;

// macOS 和 Windows 都支持系统托盘
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod tray;

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        // 注册自定义字体 - MiSans
        if let Some(font_data) = Assets::get("fonts/MiSansVF.ttf") {
            cx.text_system()
                .add_fonts(vec![font_data.data.into()])
                .expect("Failed to load MiSans font");
            println!("✓ MiSans字体加载成功");
        } else {
            eprintln!("⚠ 警告: 无法加载 MiSans 字体文件");
        }
        
        // TODO: 设置 HTTP client 以支持远程图片加载
        // 目前先使用本地头像缓存
        
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        // 在 GPUI 应用初始化之后创建系统托盘图标
        // 这样可以避免与 GPUI 的 NSApplication 初始化冲突
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            if let Some(tray_icon) = tray::create_tray_icon() {
                // 使用 Box::leak 让托盘图标在整个应用生命周期中保持存在
                // 这是必要的，因为如果托盘图标被销毁，系统托盘中的图标也会消失
                Box::leak(Box::new(tray_icon));
            }
        }

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: None,
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: point(px(100.0), px(100.0)),
                        size: size(px(1200.0), px(800.0)),
                    })),
                    kind: WindowKind::Normal,
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| App::new(window, cx));
                    // This first level on the window should be a Root.
                    cx.new(|cx| Root::new(view.into(), window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
