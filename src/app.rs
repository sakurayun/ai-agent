use gpui::*;
use gpui::prelude::FluentBuilder;

use crate::views::{home::HomeView, settings::SettingsView};
use crate::state::app_state::{AppState, Page, Theme};
use crate::components::icons::{fa, nav_icon, window_control_icon};

pub struct App {
    state: Entity<AppState>,
}

impl App {
    pub fn new(_window: &Window, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| AppState::new());
        
        Self { state }
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_page = self.state.read(cx).current_page();
        let state = self.state.clone();

        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(rgb(0x000000))
            .text_color(rgb(0xffffff))
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
                            .child(nav_icon(fa::HOUSE))
                            .text_color(match self.state.read(cx).theme() {
                                Theme::Dark => rgb(0xffffff),
                                Theme::Light => rgb(0x333333),
                            })
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let state = state.clone();
                                move |_, _, cx| {
                                    state.update(cx, |state, _| {
                                        state.set_page(Page::Home);
                                    });
                                }
                            })
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
                                if matches!(current_page, Page::Settings) {
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
                            .child(nav_icon(fa::GEAR))
                            .text_color(match self.state.read(cx).theme() {
                                Theme::Dark => rgb(0xffffff),
                                Theme::Light => rgb(0x333333),
                            })
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let state = state.clone();
                                move |_, _, cx| {
                                    state.update(cx, |state, _| {
                                        state.set_page(Page::Settings);
                                    });
                                }
                            })
                    )
                    // 填充空白，将用户区域推到底部
                    .child(div().flex_1())
                    // 用户头像区域（登录后显示）
                    .child({
                        let user = self.state.read(cx).user().cloned();
                        if let Some(user) = user {
                            let face = user.face.unwrap_or_default();
                            let pendant = user.pendant_image;
                            div()
                                .w_full()
                                .h(px(56.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .mb_2()
                                .cursor(CursorStyle::PointingHand)
                                .rounded_md()
                                .hover(|style| style.bg(match self.state.read(cx).theme() {
                                    Theme::Dark => rgb(0x1a1a1a),
                                    Theme::Light => rgb(0xe0e0e0),
                                }))
                                .child(
                                    div()
                                        .relative()
                                        .w(px(44.0))
                                        .h(px(44.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            img(face)
                                                .w(px(36.0))
                                                .h(px(36.0))
                                                .rounded_full()
                                                .object_fit(ObjectFit::Cover)
                                        )
                                        .child({
                                            if let Some(p) = pendant {
                                                img(p)
                                                    .absolute()
                                                    .top(px(-4.0))
                                                    .left(px(-4.0))
                                                    .w(px(44.0))
                                                    .h(px(44.0))
                                                    .object_fit(ObjectFit::Contain)
                                                    .into_any_element()
                                            } else { div().into_any_element() }
                                        })
                                )
                        } else {
                            div()
                        }
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
                            .justify_end()
                            .bg(match self.state.read(cx).theme() {
                                Theme::Dark => rgb(0x000000),
                                Theme::Light => rgb(0xf8f8f8),
                            })
                            .cursor(CursorStyle::Arrow)
                            // 使用 GPUI 的原生拖拽功能
                            .window_control_area(WindowControlArea::Drag)
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
                                            .text_color(match self.state.read(cx).theme() {
                                                Theme::Dark => rgb(0xffffff),
                                                Theme::Light => rgb(0x333333),
                                            })
                                            .child(window_control_icon(fa::MINUS))
                                            .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                                                println!("Minimize window");
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
                                            .text_color(match self.state.read(cx).theme() {
                                                Theme::Dark => rgb(0xffffff),
                                                Theme::Light => rgb(0x333333),
                                            })
                                            .child(window_control_icon(fa::WINDOW_MAXIMIZE))
                                            .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                                                println!("Maximize/Restore window");
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
                                            .text_color(match self.state.read(cx).theme() {
                                                Theme::Dark => rgb(0xffffff),
                                                Theme::Light => rgb(0x333333),
                                            })
                                            .child(window_control_icon(fa::XMARK))
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
                                        div().child(cx.new(|cx| HomeView::new(self.state.clone(), window, cx)))
                                    }
                                    Page::Settings => {
                                        div().child(cx.new(|cx| SettingsView::new(self.state.clone(), window, cx)))
                                    }
                                }
                            )
                    )
            )
    }
}
