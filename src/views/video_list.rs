use gpui::*;
use std::sync::Arc;
use std::path::Path;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use gpui_component::scroll::ScrollbarAxis;
use gpui_component::resizable::{h_resizable, resizable_panel};
use crate::state::app_state::{AppState, Theme, VideoInfo};

pub struct VideoListView {
    app_state: Entity<AppState>,
}

impl VideoListView {
    pub fn new(app_state: Entity<AppState>, _window: &Window, _cx: &mut Context<Self>) -> Self {
        Self { app_state }
    }
}

impl Render for VideoListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.app_state.read(cx).theme();
        let videos = self.app_state.read(cx).video_list().to_vec();
        let selected_index = self.app_state.read(cx).selected_video_index();
        
        let bg = match theme {
            Theme::Dark => rgb(0x000000),
            Theme::Light => rgb(0xffffff),
        };
        
        // 使用 GPUI 官方的 resizable 组件
        div()
            .size_full()
            .bg(bg)
            .child(
                h_resizable("video-list-layout")
            .child(
                // 左侧：视频列表 - 可调整大小
                resizable_panel()
                    .size(px(400.0))  // 初始宽度
                    .size_range(px(200.0)..px(800.0))  // 最小200px，最大800px
                    .child(self.render_video_list(videos.clone(), selected_index, theme, cx))
            )
            .child(
                // 中间：字幕内容 - 自动占据剩余空间
                resizable_panel()
                    .child(self.render_subtitle_panel(theme, cx))
            )
            .child(
                // 右侧：AI内容 - 可调整大小
                resizable_panel()
                    .size(px(320.0))  // 初始宽度
                    .size_range(px(200.0)..px(600.0))  // 最小200px，最大600px
                    .child(self.render_ai_panel(theme, cx))
            )
            )
    }
}

impl VideoListView {
    fn render_video_list(
        &self,
        videos: Vec<VideoInfo>,
        selected_index: Option<usize>,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let list_bg = match theme {
            Theme::Dark => rgb(0x0d0d0d),
            Theme::Light => rgb(0xf5f5f5),
        };
        
        div()
            .size_full() // resizable_panel 会自动管理宽度
            .flex()
            .flex_col()
            .bg(list_bg)
            .child(
                // 标题栏 - 统一高度48px
                div()
                    .w_full()
                    .h(px(48.0))
                    .flex()
                    .items_center()
                    .px_4()
                    .border_b_1()
                    .border_color(match theme {
                        Theme::Dark => rgb(0x2a2a2a),
                        Theme::Light => rgb(0xe0e0e0),
                    })
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(match theme {
                                Theme::Dark => rgb(0xffffff),
                                Theme::Light => rgb(0x333333),
                            })
                            .child(format!("视频列表 ({})", videos.len()))
                    )
            )
            .child(
                // 视频列表滚动区域 - 使用 scrollable 方法
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .children(
                        videos.into_iter().enumerate().map(|(idx, video)| {
                            self.render_video_item(video, idx, selected_index == Some(idx), theme, cx)
                        })
                    )
                    .scrollable(ScrollbarAxis::Vertical) // 添加垂直滚动条
            )
    }
    
    fn render_video_item(
        &self,
        video: VideoInfo,
        index: usize,
        is_selected: bool,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let item_bg = if is_selected {
            match theme {
                Theme::Dark => rgb(0x2a2a2a),
                Theme::Light => rgb(0xe8e8e8),
            }
        } else {
            match theme {
                Theme::Dark => rgb(0x0d0d0d),
                Theme::Light => rgb(0xf5f5f5),
            }
        };
        
        let hover_bg = match theme {
            Theme::Dark => rgb(0x1a1a1a),
            Theme::Light => rgb(0xeeeeee),
        };
        
        let text_color = match theme {
            Theme::Dark => rgb(0xffffff),
            Theme::Light => rgb(0x333333),
        };
        
        let secondary_color = match theme {
            Theme::Dark => rgb(0xaaaaaa),
            Theme::Light => rgb(0x666666),
        };
        
        // 格式化时间
        let datetime = chrono::DateTime::from_timestamp(video.pubdate, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "未知时间".to_string());
        
        let app_state = self.app_state.clone();
        
        div()
            .w_full()
            .flex()
            .flex_col()
            .p_3()
            .gap_2()
            .bg(item_bg)
            .border_b_1()
            .border_color(match theme {
                Theme::Dark => rgb(0x1a1a1a),
                Theme::Light => rgb(0xe0e0e0),
            })
            .hover(move |style| style.bg(hover_bg))
            .cursor(CursorStyle::PointingHand)
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |_view, _, _, cx| {
                app_state.update(cx, |state, _| {
                    state.set_selected_video_index(Some(index));
                });
            }))
            .child(
                // 整体布局：左右结构 - 封面在左，信息在右
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .w_full()
                    .child(
                        // 左侧：封面图 - 固定16:10比例
                        div()
                            .w(px(120.0))
                            .h(px(75.0))
                            .flex_shrink_0()
                            .rounded_md()
                            .overflow_hidden()
                            .bg(match theme {
                                Theme::Dark => rgb(0x1a1a1a),
                                Theme::Light => rgb(0xe0e0e0),
                            })
                            .child({
                                // 优先使用本地缓存的封面，否则使用网络URL
                                println!("\n========== 视频封面加载 ==========");
                                println!("[VideoList] 视频: {}", video.title);
                                
                                let pic_path = if let Some(local_path) = &video.pic_local {
                                    println!("[VideoList] ✅ 使用本地缓存: {}", local_path);
                                    local_path.clone()
                                } else {
                                    println!("[VideoList] ⚠️ 无本地缓存，使用网络URL: {}", video.pic);
                                    // 将http转为https
                                    if video.pic.starts_with("http://") {
                                        let https_url = video.pic.replace("http://", "https://");
                                        println!("[VideoList] ✅ URL转换: http -> https");
                                        https_url
                                    } else {
                                        video.pic.clone()
                                    }
                                };
                                
                                // 参考AnimatedAvatar的图片源创建逻辑
                                let is_local = Path::new(&pic_path).exists();
                                println!("[VideoList] 路径类型: {}", if is_local { "本地文件" } else { "网络URL" });
                                
                                let image_source: ImageSource = if is_local {
                                    // 本地文件路径
                                    let arc_path: Arc<Path> = Arc::from(Path::new(&pic_path));
                                    println!("[VideoList] 🔧 创建 ImageSource (本地): {:?}", arc_path);
                                    ImageSource::from(arc_path)
                                } else {
                                    // 网络URL
                                    println!("[VideoList] 🔧 创建 ImageSource (网络): {}", pic_path);
                                    ImageSource::from(pic_path.clone())
                                };
                                
                                println!("[VideoList] ✅ ImageSource 创建完成");
                                println!("===================================\n");
                                
                                // 保存theme和标题用于日志
                                let theme_for_img = theme;
                                let video_title_loading = video.title.clone();
                                let video_title_fallback = video.title.clone();
                                let pic_path_for_log = pic_path.clone();
                                
                                img(image_source)
                                    .w_full()
                                    .h_full()
                                    .object_fit(ObjectFit::Cover)
                                    .with_loading(move || {
                                        // 加载中的占位符
                                        println!("[VideoList] 📷 加载中: {}", video_title_loading);
                                        div()
                                            .w_full()
                                            .h_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .bg(match theme_for_img {
                                                Theme::Dark => rgb(0x1a1a1a),
                                                Theme::Light => rgb(0xe0e0e0),
                                            })
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(match theme_for_img {
                                                        Theme::Dark => rgb(0x666666),
                                                        Theme::Light => rgb(0x999999),
                                                    })
                                                    .child("📷")
                                            )
                                            .into_any_element()
                                    })
                                    .with_fallback(move || {
                                        // 加载失败的占位符
                                        println!("[VideoList] ❌ 加载失败: {}", video_title_fallback);
                                        println!("[VideoList] ❌ 失败的路径: {}", pic_path_for_log);
                                        div()
                                            .w_full()
                                            .h_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .bg(match theme_for_img {
                                                Theme::Dark => rgb(0x1a1a1a),
                                                Theme::Light => rgb(0xe0e0e0),
                                            })
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(match theme_for_img {
                                                        Theme::Dark => rgb(0x666666),
                                                        Theme::Light => rgb(0x999999),
                                                    })
                                                    .child("🖼️")
                                            )
                                            .into_any_element()
                                    })
                            })
                    )
                    .child(
                        // 右侧：标题 + 时间 + 统计信息（垂直排列）
                        div()
                            .flex_1()
                            .min_w_0() // 允许flex子元素收缩
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .justify_between() // 让内容均匀分布
                            .child(
                                // 标题 - 自动换行，最多显示2行
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(text_color)
                                    .line_height(relative(1.4))
                                    .overflow_hidden()
                                    .line_clamp(2) // 限制最多显示2行
                                    .child(video.title.clone())
                            )
                            .child(
                                // 发布时间
                                div()
                                    .text_xs()
                                    .text_color(secondary_color)
                                    .child(datetime)
                            )
                            .when(video.is_live_replay, |parent| {
                                // 直播回放标签
                                parent.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(rgb(0xff6b6b))
                                                .text_color(rgb(0xffffff))
                                                .text_xs()
                                                .child("🔴 直播回放")
                                        )
                                )
                            })
                    )
            )
    }
    
    fn render_subtitle_panel(&self, theme: Theme, _cx: &mut Context<Self>) -> impl IntoElement {
        let panel_bg = match theme {
            Theme::Dark => rgb(0x000000),
            Theme::Light => rgb(0xffffff),
        };
        
        let text_color = match theme {
            Theme::Dark => rgb(0xffffff),
            Theme::Light => rgb(0x333333),
        };
        
        div()
            .size_full() // resizable_panel 会自动管理大小
            .flex()
            .flex_col()
            .bg(panel_bg)
            .child(
                // 标题栏 - 统一高度48px
                div()
                    .w_full()
                    .h(px(48.0))
                    .flex()
                    .items_center()
                    .px_4()
                    .border_b_1()
                    .border_color(match theme {
                        Theme::Dark => rgb(0x2a2a2a),
                        Theme::Light => rgb(0xe0e0e0),
                    })
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child("字幕内容")
                    )
            )
            .child(
                // 内容区域
                div()
                    .flex_1()
                    .p_4()
                    .text_color(match theme {
                        Theme::Dark => rgb(0xaaaaaa),
                        Theme::Light => rgb(0x666666),
                    })
                    .child("选择视频查看字幕内容")
            )
    }
    
    fn render_ai_panel(&self, theme: Theme, _cx: &mut Context<Self>) -> impl IntoElement {
        let panel_bg = match theme {
            Theme::Dark => rgb(0x0d0d0d),
            Theme::Light => rgb(0xf5f5f5),
        };
        
        let text_color = match theme {
            Theme::Dark => rgb(0xffffff),
            Theme::Light => rgb(0x333333),
        };
        
        div()
            .size_full() // resizable_panel 会自动管理宽度
            .flex()
            .flex_col()
            .bg(panel_bg)
            .child(
                // 标题栏 - 统一高度48px
                div()
                    .w_full()
                    .h(px(48.0))
                    .flex()
                    .items_center()
                    .px_4()
                    .border_b_1()
                    .border_color(match theme {
                        Theme::Dark => rgb(0x2a2a2a),
                        Theme::Light => rgb(0xe0e0e0),
                    })
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child("AI 分析")
                    )
            )
            .child(
                // 内容区域
                div()
                    .flex_1()
                    .p_4()
                    .text_color(match theme {
                        Theme::Dark => rgb(0xaaaaaa),
                        Theme::Light => rgb(0x666666),
                    })
                    .child("AI 内容功能开发中...")
            )
    }
}
