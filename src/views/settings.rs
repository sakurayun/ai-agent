use crate::state::app_state::{AppState, Theme};
use gpui::*;
use gpui_component::button::ButtonVariants;
use gpui_component::*;

pub struct SettingsView {
    input_state: Entity<input::InputState>,
    app_state: Entity<AppState>,
}

impl SettingsView {
    pub fn new(app_state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            input::InputState::new(window, cx)
                .placeholder("Enter your name...")
                .default_value("GPUI User")
        });

        Self {
            input_state,
            app_state,
        }
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.app_state.read(cx).theme();
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(match theme {
                Theme::Dark => rgb(0x000000),
                Theme::Light => rgb(0xffffff),
            })
            .text_color(match theme {
                Theme::Dark => rgb(0xffffff),
                Theme::Light => rgb(0x000000),
            })
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .mb_2()
                    .child("Settings"),
            )
            .child(
                div()
                    .text_base()
                    .text_color(match theme {
                        Theme::Dark => rgb(0xaaaaaa),
                        Theme::Light => rgb(0x666666),
                    })
                    .child("Configure your application preferences."),
            )
            .child(
                div()
                    .mt_6()
                    .p_6()
                    .rounded_lg()
                    .border_1()
                    .border_color(match theme {
                        Theme::Dark => rgb(0x333333),
                        Theme::Light => rgb(0xcccccc),
                    })
                    .bg(match theme {
                        Theme::Dark => rgb(0x0d0d0d),
                        Theme::Light => rgb(0xf5f5f5),
                    })
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .mb_2()
                            .child("User Information"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Name"),
                            )
                            .child(self.input_state.clone()),
                    )
                    .child(
                        div().mt_4().child(
                            button::Button::new("save")
                                .primary()
                                .label("Save Settings")
                                .on_click({
                                    let input_state = self.input_state.clone();
                                    move |_, _, cx| {
                                        let value = input_state.read(cx).text();
                                        println!("Settings saved! Name: {}", value);
                                    }
                                }),
                        ),
                    ),
            )
            .child(
                div()
                    .mt_4()
                    .p_6()
                    .rounded_lg()
                    .border_1()
                    .border_color(match theme {
                        Theme::Dark => rgb(0x333333),
                        Theme::Light => rgb(0xcccccc),
                    })
                    .bg(match theme {
                        Theme::Dark => rgb(0x0d0d0d),
                        Theme::Light => rgb(0xf5f5f5),
                    })
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .mb_1()
                            .child("Appearance"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(match theme {
                                Theme::Dark => rgb(0xaaaaaa),
                                Theme::Light => rgb(0x666666),
                            })
                            .mb_2()
                            .child("Tip: Click the logo to quickly toggle theme"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(if theme == Theme::Light {
                                button::Button::new("theme-light")
                                    .primary() // 高亮当前选中的主题
                                    .label("Light ✓")
                                    .on_click({
                                        let app_state = self.app_state.clone();
                                        move |_, _, cx| {
                                            app_state.update(cx, |state, _| {
                                                state.set_theme(Theme::Light);
                                            });
                                        }
                                    })
                            } else {
                                button::Button::new("theme-light")
                                    .outline()
                                    .label("Light")
                                    .on_click({
                                        let app_state = self.app_state.clone();
                                        move |_, _, cx| {
                                            app_state.update(cx, |state, _| {
                                                state.set_theme(Theme::Light);
                                            });
                                        }
                                    })
                            })
                            .child(if theme == Theme::Dark {
                                button::Button::new("theme-dark")
                                    .primary() // 高亮当前选中的主题
                                    .label("Dark ✓")
                                    .on_click({
                                        let app_state = self.app_state.clone();
                                        move |_, _, cx| {
                                            app_state.update(cx, |state, _| {
                                                state.set_theme(Theme::Dark);
                                            });
                                        }
                                    })
                            } else {
                                button::Button::new("theme-dark")
                                    .outline()
                                    .label("Dark")
                                    .on_click({
                                        let app_state = self.app_state.clone();
                                        move |_, _, cx| {
                                            app_state.update(cx, |state, _| {
                                                state.set_theme(Theme::Dark);
                                            });
                                        }
                                    })
                            }),
                    ),
            )
            .child(
                div()
                    .mt_4()
                    .p_6()
                    .rounded_lg()
                    .border_1()
                    .border_color(match theme {
                        Theme::Dark => rgb(0x333333),
                        Theme::Light => rgb(0xcccccc),
                    })
                    .bg(match theme {
                        Theme::Dark => rgb(0x0d0d0d),
                        Theme::Light => rgb(0xf5f5f5),
                    })
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("About"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(match theme {
                                Theme::Dark => rgb(0xaaaaaa),
                                Theme::Light => rgb(0x666666),
                            })
                            .child("Version: 0.1.0"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(match theme {
                                Theme::Dark => rgb(0xaaaaaa),
                                Theme::Light => rgb(0x666666),
                            })
                            .child("Built with Rust and GPUI"),
                    ),
            )
    }
}
