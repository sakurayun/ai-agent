use gpui::*;
use gpui_component::*;

mod app;
mod views;
mod state;
mod components;
mod utils;
mod assets;

use app::App;
use assets::Assets;

#[cfg(target_os = "windows")]
mod tray;

fn main() {
    // 在Windows上创建系统托盘图标
    #[cfg(target_os = "windows")]
    let _tray_icon = tray::create_tray_icon();
    
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

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
