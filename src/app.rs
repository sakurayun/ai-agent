use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::IconName;

use crate::views::{home::HomeView, settings::SettingsView, video_list::VideoListView};
use crate::state::app_state::{AppState, Page, Theme};
use crate::components::AnimatedAvatar;

pub struct App {
    state: Entity<AppState>,
    _animation_driver: Option<()>, // 占位符，实际的定时器在启动时创建
    // 缓存视图实例，避免每次渲染都重新创建
    home_view: Option<Entity<HomeView>>,
    settings_view: Option<Entity<SettingsView>>,
    video_list_view: Option<Entity<VideoListView>>,
}

impl App {
    pub fn new(_window: &Window, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| AppState::new());
        
        // 启动全局动画驱动器 - 60fps持续刷新
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(std::time::Duration::from_millis(16)).await;
                
                // 强制触发窗口刷新
                let result = this.update(cx, |_, cx| {
                    cx.notify(); // 通知App重绘
                });
                
                // 如果App被销毁，退出循环
                if result.is_err() {
                    break;
                }
            }
        }).detach();
        
        Self { 
            state,
            _animation_driver: Some(()),
            home_view: None,
            settings_view: None,
            video_list_view: None,
        }
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_page = self.state.read(cx).current_page();
        let state = self.state.clone();
        let is_menu_open = self.state.read(cx).is_user_menu_open();
        let theme = self.state.read(cx).theme();

        div()
            .size_full()
            .relative()
            .flex()
            .flex_row()
            .bg(rgb(0x000000))
            .text_color(rgb(0xffffff))
            .font_family("MiSans VF") // 全局默认字体
            .child(
                // Sidebar - DRAGGABLE REGION: Click and drag here to move the window
                div()
                    .id("sidebar-drag-region")
                    .w(px(60.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_2()
                    .bg(match self.state.read(cx).theme() {
                        Theme::Dark => rgb(0x0d0d0d),
                        Theme::Light => rgb(0xf0f0f0),
                    })
                    .cursor(CursorStyle::Arrow)
                    // 使用 GPUI 的原生拖拽功能
                    .window_control_area(WindowControlArea::Drag)
                    // Logo 容器
                    .child(
                        div()
                            .w_full()
                            .h(px(56.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .mb_2()
                            .child(
                                // 根据主题切换logo - 可点击切换主题
                                div()
                                    .cursor(CursorStyle::PointingHand)
                                    .rounded_md()
                                    .p_1()
                                    .hover(|style| style.bg(match self.state.read(cx).theme() {
                                        Theme::Dark => rgb(0x1a1a1a),
                                        Theme::Light => rgb(0xe0e0e0),
                                    }))
                                    .child({
                                        // 深色主题使用白色logo，浅色主题使用黑色logo
                                        let theme_for_logo = self.state.read(cx).theme();
                                        let logo = match theme_for_logo {
                                            Theme::Dark => img("assets/logo-white.png"),
                                            Theme::Light => img("assets/logo-black.png"),
                                        };

                                        logo
                                            .w(px(40.0))
                                            .object_fit(ObjectFit::Contain)
                                            .with_loading(|| {
                                                div()
                                                    .w(px(40.0))
                                                    .rounded_sm()
                                                    .bg(black().opacity(0.08))
                                                    .into_any_element()
                                            })
                                            .with_fallback(move || {
                                                div()
                                                    .w(px(40.0))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .rounded_sm()
                                                    .border_1()
                                                    .border_color(match theme_for_logo {
                                                        Theme::Dark => rgb(0x333333),
                                                        Theme::Light => rgb(0xcccccc),
                                                    })
                                                    .text_sm()
                                                    .text_color(match theme_for_logo {
                                                        Theme::Dark => rgb(0xaaaaaa),
                                                        Theme::Light => rgb(0x666666),
                                                    })
                                                    .child("?")
                                                    .into_any_element()
                                            })
                                    })
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let state = self.state.clone();
                                        move |_, _, cx| {
                                            state.update(cx, |state, _| {
                                                state.toggle_theme();
                                            });
                                        }
                                    })
                            )
                    )
                    .child(
                        div()
                            .w_full()
                            .h(px(48.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_lg()
                            .rounded_md()
                            .map(|div| {
                                let theme = self.state.read(cx).theme();
                                if matches!(current_page, Page::Home) {
                                    div.bg(match theme {
                                        Theme::Dark => rgb(0x2a2a2a),
                                        Theme::Light => rgb(0xcccccc),
                                    })
                                } else {
                                    div.hover(|style| style.bg(match theme {
                                        Theme::Dark => rgb(0x1a1a1a),
                                        Theme::Light => rgb(0xdddddd),
                                    }))
                                }
                            })
                            .child(
                                div()
                                    .text_color(match self.state.read(cx).theme() {
                                        Theme::Dark => rgb(0xffffff),
                                        Theme::Light => rgb(0x333333),
                                    })
                                    .child(IconName::LayoutDashboard)
                            )
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let state = state.clone();
                                move |_, _, cx| {
                                    state.update(cx, |state, _| {
                                        state.set_page(Page::Home);
                                    });
                                }
                            })
                    )
                    // 填充空白，将用户区域推到底部
                    .child(div().flex_1())
                    // 用户登录状态区域（始终显示）
                    .child({
                        let user = self.state.read(cx).user().cloned();
                        let is_logged_in = self.state.read(cx).is_logged_in();
                        let avatar_theme = self.state.read(cx).theme();
                        
                        // 用户头像/图标按钮
                        div()
                            .id("user-avatar-button")
                            .w_full()
                            .h(px(56.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .mb_2()
                            .cursor(CursorStyle::PointingHand)
                            .child({
                                if is_logged_in && user.is_some() {
                                    let user = user.unwrap();
                                    
                                    // 获取头像路径
                                    let avatar_path = if let Some(local_path) = &user.face_local {
                                        local_path.clone()
                                    } else if let Some(face_url) = &user.face {
                                        face_url.clone()
                                    } else {
                                        String::new()
                                    };
                                    
                                    // 使用AnimatedAvatar组件支持动画webp
                                    cx.new(|_| AnimatedAvatar::new(avatar_path, px(40.0)))
                                        .into_any_element()
                                } else {
                                    // 未登录时显示用户图标
                                    div()
                                        .w(px(40.0))
                                        .h(px(40.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_full()
                                        .bg(match avatar_theme {
                                            Theme::Dark => rgb(0x2a2a2a),
                                            Theme::Light => rgb(0xdddddd),
                                        })
                                        .child(
                                            div()
                                                .text_color(match avatar_theme {
                                                    Theme::Dark => rgb(0xaaaaaa),
                                                    Theme::Light => rgb(0x666666),
                                                })
                                                .child(IconName::User)
                                        )
                                        .into_any_element()
                                }
                            })
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let state = self.state.clone();
                                move |_, _, cx| {
                                    state.update(cx, |state, _| {
                                        state.toggle_user_menu();
                                    });
                                }
                            })
                    })
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(
                        // Custom titlebar - DRAGGABLE REGION: Click and drag here to move the window
                        div()
                            .id("titlebar-drag-region")
                            .h(px(32.0))
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between() // 改为两端对齐
                            .bg(match self.state.read(cx).theme() {
                                Theme::Dark => rgb(0x080808), // 深色：主背景(0x000000) < 控制栏(0x080808) < 侧边栏(0x0d0d0d)
                                Theme::Light => rgb(0xf8f8f8),
                            })
                            .cursor(CursorStyle::Arrow)
                            // 使用 GPUI 的原生拖拽功能
                            .window_control_area(WindowControlArea::Drag)
                            // 左侧：头像和名称
                            .child({
                                let is_logged_in = self.state.read(cx).is_logged_in();
                                let user = self.state.read(cx).user();
                                
                                if is_logged_in && user.is_some() {
                                    let user = user.unwrap();
                                    
                                    // 获取头像路径
                                    let avatar_path = if let Some(local_path) = &user.face_local {
                                        local_path.clone()
                                    } else if let Some(face_url) = &user.face {
                                        face_url.clone()
                                    } else {
                                        String::new()
                                    };
                                    
                                    let uname = user.uname.clone().unwrap_or_else(|| "用户".to_string());
                                    
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .pl_3()
                                        .child(
                                            // 使用AnimatedAvatar组件（20px小尺寸）
                                            cx.new(|_| AnimatedAvatar::new(avatar_path, px(20.0)))
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(match self.state.read(cx).theme() {
                                                    Theme::Dark => rgb(0xffffff),
                                                    Theme::Light => rgb(0x333333),
                                                })
                                                .child(uname)
                                        )
                                        .into_any_element()
                                } else {
                                    // 未登录时显示空div
                                    div().into_any_element()
                                }
                            })
                            // 右侧：窗口控制按钮
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .h_full()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(px(48.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .hover(|style| style.bg(match self.state.read(cx).theme() {
                                                Theme::Dark => rgb(0x333333),
                                                Theme::Light => rgb(0xdddddd),
                                            }))
                                            .child(
                                                div()
                                                    .text_color(match self.state.read(cx).theme() {
                                                        Theme::Dark => rgb(0xffffff),
                                                        Theme::Light => rgb(0x333333),
                                                    })
                                                    .child(IconName::Minus)
                                            )
                                            .on_mouse_down(gpui::MouseButton::Left, |_, window, cx| {
                                                // 最小化窗口
                                                window.minimize_window();
                                                cx.stop_propagation();
                                            })
                                    )
                                    .child(
                                        div()
                                            .h_full()
                                            .w(px(48.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .hover(|style| style.bg(match self.state.read(cx).theme() {
                                                Theme::Dark => rgb(0x333333),
                                                Theme::Light => rgb(0xdddddd),
                                            }))
                                            .child(
                                                div()
                                                    .text_color(match self.state.read(cx).theme() {
                                                        Theme::Dark => rgb(0xffffff),
                                                        Theme::Light => rgb(0x333333),
                                                    })
                                                    .child(IconName::WindowMaximize)
                                            )
                                            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                                                // 切换全屏模式（类似最大化效果）
                                                window.toggle_fullscreen();
                                                cx.stop_propagation();
                                            })
                                    )
                                    .child(
                                        div()
                                            .h_full()
                                            .w(px(48.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .hover(|style| style.bg(rgb(0xc42b1c)))
                                            .child(
                                                div()
                                                    .text_color(match self.state.read(cx).theme() {
                                                        Theme::Dark => rgb(0xffffff),
                                                        Theme::Light => rgb(0x333333),
                                                    })
                                                    .child(IconName::WindowClose)
                                            )
                                            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                                cx.quit();
                                            })
                                    )
                            )
                    )
                    .child(
                        // Main content area
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .bg(match self.state.read(cx).theme() {
                                Theme::Dark => rgb(0x000000),
                                Theme::Light => rgb(0xffffff),
                            })
                            .child(
                                match current_page {
                                    Page::Home => {
                                        // 缓存 HomeView 实例，避免每次渲染都重新创建
                                        if self.home_view.is_none() {
                                            println!("🎯 [App] 首次创建 HomeView");
                                            self.home_view = Some(cx.new(|cx| HomeView::new(self.state.clone(), window, cx)));
                                        }
                                        div()
                                            .size_full()
                                            .child(self.home_view.clone().unwrap())
                                    }
                                    Page::VideoList => {
                                        // 缓存 VideoListView 实例
                                        if self.video_list_view.is_none() {
                                            println!("📹 [App] 首次创建 VideoListView");
                                            self.video_list_view = Some(cx.new(|cx| VideoListView::new(self.state.clone(), window, cx)));
                                        }
                                        div()
                                            .size_full()
                                            .child(self.video_list_view.clone().unwrap())
                                    }
                                    Page::Settings => {
                                        // 缓存 SettingsView 实例
                                        if self.settings_view.is_none() {
                                            println!("⚙️  [App] 首次创建 SettingsView");
                                            self.settings_view = Some(cx.new(|cx| SettingsView::new(self.state.clone(), window, cx)));
                                        }
                                        div()
                                            .size_full()
                                            .child(self.settings_view.clone().unwrap())
                                    }
                                }
                            )
                    )
            )
            // 点击外部关闭菜单的遮罩层
            .when(is_menu_open, |parent| {
                parent.child(
                    div()
                        .absolute()
                        .size_full()
                        .top_0()
                        .left_0()
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let state = self.state.clone();
                            move |_, _, cx| {
                                state.update(cx, |state, _| {
                                    state.set_user_menu_open(false);
                                });
                            }
                        })
                )
            })
            // 悬浮菜单层 - 完全独立于侧边栏布局
            .child({
                if is_menu_open {
                    
                    div()
                        .absolute()
                        .bottom(px(72.0))  // 距离底部72px，留出头像按钮的空间
                        .left(px(8.0))
                        .w(px(180.0))
                        .bg(match theme {
                            Theme::Dark => rgb(0x1a1a1a),
                            Theme::Light => rgb(0xffffff),
                        })
                        .rounded_md()
                        .shadow_lg()
                        .py_2()
                        // 设置选项
                        .child(
                            div()
                                .w_full()
                                .px_4()
                                .py_3()
                                .flex()
                                .items_center()
                                .gap_3()
                                .cursor(CursorStyle::PointingHand)
                                .hover(|style| style.bg(match theme {
                                    Theme::Dark => rgb(0x2a2a2a),
                                    Theme::Light => rgb(0xf0f0f0),
                                }))
                                .child(
                                    div()
                                        .text_color(match theme {
                                            Theme::Dark => rgb(0xffffff),
                                            Theme::Light => rgb(0x333333),
                                        })
                                        .child(IconName::Settings)
                                )
                                .child(
                                    div()
                                        .text_color(match theme {
                                            Theme::Dark => rgb(0xffffff),
                                            Theme::Light => rgb(0x333333),
                                        })
                                        .child("设置")
                                )
                                .on_mouse_down(gpui::MouseButton::Left, {
                                    let state = self.state.clone();
                                    move |_, _, cx| {
                                        state.update(cx, |state, _| {
                                            state.set_page(Page::Settings);
                                            state.set_user_menu_open(false);
                                        });
                                    }
                                })
                        )
                        .into_any_element()
                } else {
                    div().into_any_element()
                }
            })
    }
}
