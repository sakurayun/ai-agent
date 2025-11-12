use std::sync::{Arc, OnceLock};
use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::*;
use crate::state::app_state::{AppState, Theme, Cookies, UserProfile, VideoInfo, Page};
use qrcode::QrCode;
use qrcode::render::svg;
use gpui_component::input::{InputState, InputEvent};

// Bilibili API 的标准 User-Agent
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

// 全局 Tokio runtime，参考 Zed 的实现
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_runtime_handle() -> tokio::runtime::Handle {
    tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
        let runtime = RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to initialize Tokio runtime")
        });
        runtime.handle().clone()
    })
}

pub struct HomeView {
    app_state: Entity<AppState>,
    search_input: Entity<InputState>,
}

impl HomeView {
    pub fn new(app_state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 创建输入框状态，设置占位符
        let search_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("搜索你感兴趣的内容...")
        });
        
        // 订阅输入事件 - 增加详细日志
        let app_state_clone = app_state.clone();
        
        println!("🎯 [HomeView::new] 创建输入框并订阅事件");
        
        cx.subscribe_in(&search_input, window, move |view, state, event, _window, cx| {
            let event_name = match event {
                InputEvent::Change => "Change",
                InputEvent::PressEnter { .. } => "PressEnter",
                InputEvent::Focus => "Focus",
                InputEvent::Blur => "Blur",
            };
            println!("📨 [InputEvent] 收到输入事件: {}", event_name);
            
            match event {
                InputEvent::Change => {
                    // 输入内容改变时，更新到 AppState
                    let text = state.read(cx).value().to_string();
                    println!("✏️  [InputEvent::Change] 输入内容改变: '{}'", text);
                    app_state_clone.update(cx, |s, _| {
                        s.set_search_text(text.clone());
                    });
                    println!("💾 [InputEvent::Change] 已保存到 AppState");
                }
                InputEvent::PressEnter { secondary } => {
                    // 按下 Enter 键时触发搜索
                    println!("⌨️  [InputEvent::PressEnter] 按下 Enter 键，secondary: {}", secondary);
                    let current_text = state.read(cx).value().to_string();
                    println!("📝 [InputEvent::PressEnter] 当前输入内容: '{}'", current_text);
                    Self::trigger_search(view, cx);
                }
                InputEvent::Focus => {
                    println!("🎯 [InputEvent::Focus] 搜索框获得焦点");
                    let current_text = state.read(cx).value().to_string();
                    println!("📝 [InputEvent::Focus] 当前输入内容: '{}'", current_text);
                }
                InputEvent::Blur => {
                    println!("💤 [InputEvent::Blur] 搜索框失去焦点");
                    let current_text = state.read(cx).value().to_string();
                    println!("📝 [InputEvent::Blur] 当前输入内容: '{}'", current_text);
                }
            }
        }).detach();
        
        Self { 
            app_state,
            search_input,
        }
    }

    fn trigger_search(view: &mut Self, cx: &mut Context<Self>) {
        println!("🚀 [trigger_search] 进入搜索函数");
        
        // 读取搜索框内容
        let search_text = view.search_input.read_with(cx, |state, _| {
            let value = state.value().to_string();
            println!("📖 [trigger_search] 从 InputState 读取到的值: '{}'", value);
            value
        });
        
        println!("🔍 [trigger_search] 开始搜索，内容: '{}'", search_text);
        
        // 获取当前登录用户的 UID
        let uid = view.app_state.read_with(cx, |s, _| {
            s.user().and_then(|u| u.uname.clone())
        });
        
        if uid.is_none() {
            println!("❌ 用户未登录，无法搜索");
            return;
        }
        
        // 保存搜索文本到状态
        view.app_state.update(cx, |s, _| {
            s.set_search_text(search_text.clone());
        });
        
        // 获取 Cookie
        let cookie = view.app_state.read_with(cx, |s, _| {
            s.cookie_header().unwrap_or_default()
        });
        
        // 临时使用固定的 mid 进行测试
        let mid = "3461574394120551"; // 测试用的 mid
        
        println!("🚀 开始获取 UID {} 的视频合集列表", mid);
        
        // 在后台线程中执行 API 调用
        let app_state_for_update = view.app_state.clone();
        
        cx.spawn(async move |_: WeakEntity<HomeView>, mut _cx| {
            let handle = get_runtime_handle();
            let cookie_for_collections = cookie.clone();
            
            // 使用 Tokio runtime 执行异步请求
            let result = handle.spawn(async move {
                crate::api::bilibili::fetch_space_collections(
                    mid,
                    &cookie_for_collections,
                    1,
                    20
                ).await
            }).await;
            
            match result {
                Ok(Ok(data)) => {
                    println!("\n✅ 成功获取合集列表！");
                    
                    // 显示合集信息
                    if let Some(seasons) = &data.items_lists.seasons_list {
                        println!("\n📚 合集列表:");
                        for season in seasons {
                            println!("  - {} (ID: {}, 共{}个视频)", 
                                season.meta.name,
                                season.meta.season_id,
                                season.meta.total
                            );
                            
                            // 获取第一个合集的视频列表（测试）
                            let season_id = season.meta.season_id.to_string();
                            let mid = mid.to_string();
                            let cookie = cookie.clone();
                            let app_state_clone = app_state_for_update.clone();
                            
                            println!("\n🔍 正在获取合集 {} 的视频列表...", season.meta.name);
                            
                            let videos_result = handle.spawn(async move {
                                crate::api::bilibili::fetch_all_season_archives(
                                    &mid,
                                    &season_id,
                                    &cookie
                                ).await
                            }).await;
                            
                            match videos_result {
                                Ok(Ok(videos)) => {
                                    println!("\n✅ 成功获取 {} 个视频！", videos.len());
                                    
                                    // 转换为 VideoInfo 格式并下载封面
                                    println!("\n📥 开始下载视频封面...");
                                    let video_list: Vec<VideoInfo> = videos.iter().map(|v| {
                                        let is_live_replay = v.title.contains("【直播回放】") || 
                                                            v.title.contains("直播回放");
                                        
                                        // 下载封面到本地
                                        let pic_url = if v.pic.starts_with("http://") {
                                            v.pic.replace("http://", "https://")
                                        } else {
                                            v.pic.clone()
                                        };
                                        
                                        let pic_local = match crate::utils::download_cover(&pic_url) {
                                            Ok(path_arc) => {
                                                let path_str = path_arc.display().to_string();
                                                println!("[HomeView] ✅ 封面下载成功: {}", v.title);
                                                Some(path_str)
                                            }
                                            Err(e) => {
                                                println!("[HomeView] ❌ 封面下载失败: {} - {}", v.title, e);
                                                None
                                            }
                                        };
                                        
                                        VideoInfo {
                                            aid: v.aid,
                                            bvid: v.bvid.clone(),
                                            title: v.title.clone(),
                                            pic: v.pic.clone(),
                                            pic_local,
                                            description: None,
                                            pubdate: v.pubdate,
                                            duration: v.duration,
                                            view_count: v.stat.view,
                                            like_count: v.stat.like.unwrap_or(0),
                                            is_live_replay,
                                        }
                                    }).collect();
                                    
                                    println!("✅ 所有封面下载完成！");
                                    
                                    // 统计直播回放数量
                                    let live_replay_count = video_list.iter()
                                        .filter(|v| v.is_live_replay)
                                        .count();
                                    
                                    println!("\n📊 统计:");
                                    println!("  总视频数: {}", video_list.len());
                                    println!("  直播回放: {} 个 🔴", live_replay_count);
                                    println!("  普通视频: {} 个 ⚪", video_list.len() - live_replay_count);
                                    
                                    // 保存到状态并跳转到视频列表页面
                                    let _ = _cx.update(|cx| {
                                        app_state_clone.update(cx, |state, _| {
                                            state.set_video_list(video_list);
                                            state.set_selected_video_index(None);
                                            state.set_page(Page::VideoList);
                                        })
                                    });
                                    
                                    println!("\n🎉 已跳转到视频列表页面");
                                },
                                Ok(Err(e)) => {
                                    println!("❌ 获取视频列表失败: {}", e);
                                },
                                Err(e) => {
                                    println!("❌ 任务执行失败: {}", e);
                                }
                            }
                            
                            // 只获取第一个合集进行测试
                            break;
                        }
                    }
                    
                    // 显示系列信息
                    if let Some(series) = &data.items_lists.series_list {
                        println!("\n📖 系列列表:");
                        for s in series {
                            println!("  - {} (ID: {}, 共{}个视频)", 
                                s.meta.name,
                                s.meta.series_id,
                                s.meta.total
                            );
                        }
                    }
                },
                Ok(Err(e)) => {
                    println!("❌ API 调用失败: {}", e);
                },
                Err(e) => {
                    println!("❌ 任务执行失败: {}", e);
                }
            }
            
            Ok::<(), anyhow::Error>(())
        }).detach();
    }
    
    fn start_qr_flow(app_state: Entity<AppState>, cx: &mut Context<Self>) {
        app_state.update(cx, |s, _| { s.set_qr_started(true); s.set_qr_status("正在获取二维码..."); });
        Self::request_qr(app_state, cx);
    }

    fn request_qr(app_state: Entity<AppState>, cx: &mut Context<Self>) {
        // 获取 Tokio runtime handle
        let handle = get_runtime_handle();
        
        cx.spawn(async move |_: WeakEntity<HomeView>, cx: &mut AsyncApp| {
            // 1) 获取二维码
            let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
            println!("\n========== API Request ==========");
            println!("Method: GET");
            println!("URL: {}", url);
            println!("Headers:");
            println!("  User-Agent: {}", USER_AGENT);
            println!("Body: None");
            println!("=================================\n");
            
            let client = reqwest::Client::new();
            // 使用 Tokio runtime handle 执行异步请求
            let response = handle.spawn(async move {
                client
                    .get(url)
                    .header("User-Agent", USER_AGENT)
                    .send()
                    .await
            }).await??;
            
            println!("\n========== API Response ==========");
            println!("URL: {}", url);
            println!("Status: {}", response.status());
            println!("Response Headers:");
            for (key, value) in response.headers().iter() {
                if let Ok(val_str) = value.to_str() {
                    println!("  {}: {}", key, val_str);
                }
            }
            
            let body = response.text().await?;
            println!("Response Body: {}", body);
            println!("==================================\n");
            
            #[derive(serde::Deserialize)]
            struct GenerateResp { code: i64, data: Option<GenData> }
            #[derive(serde::Deserialize)]
            struct GenData { url: String, qrcode_key: String }
            let parsed: GenerateResp = serde_json::from_str(&body)?;
            if parsed.code != 0 { anyhow::bail!("生成二维码失败"); }
            let gen = parsed.data.unwrap();

            // 2) 生成二维码SVG并作为 Image(SVG) 显示
            let code = QrCode::new(gen.url.as_bytes())?;
            let svg_text = code
                .render::<svg::Color>()
                .min_dimensions(256, 256)
                .quiet_zone(true)
                .build();
            let svg_bytes = svg_text.into_bytes();
            app_state.update(cx, |s, _| {
                s.set_qrcode_key(Some(gen.qrcode_key.clone()));
                s.set_qr_svg(Some(svg_bytes));
                s.set_qr_status("请使用手机客户端扫码并确认");
            })?;

            // 3) 开始轮询
            use std::time::Duration;
            let client = reqwest::Client::builder()
                .cookie_store(true)
                .build()?;
            let start = std::time::Instant::now();
            let handle_clone = handle.clone();
            
            loop {
                if start.elapsed() > Duration::from_secs(180) {
                    app_state.update(cx, |s, _| { s.set_qr_status("二维码已超时，请刷新"); })?;
                    break;
                }

                let qrcode_key = app_state.read_with(cx, |s, _| s.qrcode_key().cloned())?;
                if qrcode_key.is_none() { break; }
                let key = qrcode_key.unwrap();

                let url = format!("https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={}", key);
                
                println!("\n========== API Request ==========");
                println!("Method: GET");
                println!("URL: {}", url);
                println!("Headers:");
                println!("  User-Agent: {}", USER_AGENT);
                println!("Body: None");
                println!("=================================\n");
                
                let response = handle_clone.spawn({
                    let client = client.clone();
                    let url = url.clone();
                    async move {
                        client
                            .get(&url)
                            .header("User-Agent", USER_AGENT)
                            .send()
                            .await
                    }
                }).await??;
                
                println!("\n========== API Response ==========");
                println!("URL: {}", url);
                println!("Status: {}", response.status());
                println!("Response Headers:");
                for (key, value) in response.headers().iter() {
                    if let Ok(val_str) = value.to_str() {
                        println!("  {}: {}", key, val_str);
                    }
                }
                
                let headers = response.headers().clone();
                let body = response.text().await?;
                println!("Response Body: {}", body);
                println!("==================================\n");

                #[derive(serde::Deserialize)]
                struct PollData { code: i64 }
                #[derive(serde::Deserialize)]
                struct PollResp { code: i64, data: Option<PollData> }
                let parsed: PollResp = serde_json::from_str(&body).unwrap_or(PollResp{ code: -1, data: None });
                if parsed.code != 0 {
                    handle_clone.spawn(async {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }).await?;
                    continue;
                }
                let Some(data) = parsed.data else {
                    handle_clone.spawn(async {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }).await?;
                    continue;
                };

                match data.code {
                    0 => {
                        // 登录成功：解析 Cookie
                        let mut cookies = Cookies::default();
                        for (_k, v) in headers.iter().filter(|(k, _)| k.as_str().eq_ignore_ascii_case("set-cookie")) {
                            if let Ok(line) = v.to_str() {
                                // 解析 Set-Cookie 头，提取第一个键值对
                                if let Some(first_part) = line.split(';').next() {
                                    if let Some((k, v)) = first_part.split_once('=') {
                                        let k = k.trim();
                                        let v = v.trim();
                                        match k {
                                            "SESSDATA" => cookies.SESSDATA = v.to_string(),
                                            "DedeUserID" => cookies.DedeUserID = Some(v.to_string()),
                                            "DedeUserID__ckMd5" => cookies.DedeUserID__ckMd5 = Some(v.to_string()),
                                            "bili_jct" => cookies.bili_jct = Some(v.to_string()),
                                            "sid" => cookies.sid = Some(v.to_string()),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }

                        app_state.update(cx, |s, _| {
                            s.set_cookies(cookies);
                            s.set_logged_in(true);
                            s.persist_login();
                            s.set_qr_status("登录成功");
                        })?;

                        // 获取用户信息
                        println!("登录成功，正在获取用户信息并设置测试头像...");
                        Self::fetch_user_info(app_state.clone(), cx).await.ok();
                        break;
                    }
                    86038 => {
                        app_state.update(cx, |s, _| {
                            s.set_qr_status("二维码已失效，正在刷新...");
                            s.set_qr_started(false);
                            s.set_qr_svg(None);
                            s.set_qrcode_key(None);
                        })?;
                        // 重新生成
                        let refresh_url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
                        println!("\n========== API Request (Refresh QR) ==========");
                        println!("Method: GET");
                        println!("URL: {}", refresh_url);
                        println!("Headers:");
                        println!("  User-Agent: {}", USER_AGENT);
                        println!("Body: None");
                        println!("==============================================\n");
                        
                        let response = handle_clone.spawn({
                            let client = client.clone();
                            async move {
                                client
                                    .get(refresh_url)
                                    .header("User-Agent", USER_AGENT)
                                    .send()
                                    .await
                            }
                        }).await??;
                        
                        println!("\n========== API Response (Refresh QR) ==========");
                        println!("URL: {}", refresh_url);
                        println!("Status: {}", response.status());
                        println!("Response Headers:");
                        for (key, value) in response.headers().iter() {
                            if let Ok(val_str) = value.to_str() {
                                println!("  {}: {}", key, val_str);
                            }
                        }
                        
                        let body = response.text().await?;
                        println!("Response Body: {}", body);
                        println!("===============================================\n");
                        let parsed: GenerateResp = serde_json::from_str(&body)?;
                        if parsed.code != 0 { anyhow::bail!("生成二维码失败"); }
                        let gen = parsed.data.unwrap();
                        let code = QrCode::new(gen.url.as_bytes())?;
                        let svg_text = code.render::<svg::Color>().min_dimensions(256, 256).quiet_zone(true).build();
                        let svg_bytes = svg_text.into_bytes();
                        app_state.update(cx, |s, _| {
                            s.set_qrcode_key(Some(gen.qrcode_key.clone()));
                            s.set_qr_svg(Some(svg_bytes));
                            s.set_qr_status("请使用手机客户端扫码并确认");
                        })?;
                    }
                    86090 => {
                        app_state.update(cx, |s, _| { s.set_qr_status("已扫码，等待确认..."); })?;
                    }
                    86101 => {
                        // 未扫码
                    }
                    _ => {}
                }
                handle_clone.spawn(async {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }).await?;
            }
            Ok::<(), anyhow::Error>(())
        }).detach();
    }

    async fn fetch_user_info(app_state: Entity<AppState>, cx: &mut AsyncApp) -> anyhow::Result<()> {
        let cookie_header = app_state.read_with(cx, |s, _| s.cookie_header())?;
        let Some(cookie) = cookie_header else { return Ok(()); };
        
        let url = "https://api.bilibili.com/x/web-interface/nav";
        println!("\n========== API Request (User Info) ==========");
        println!("Method: GET");
        println!("URL: {}", url);
        println!("Request Headers:");
        println!("  User-Agent: {}", USER_AGENT);
        println!("  Cookie: {}", cookie);
        println!("Body: None");
        println!("=============================================\n");
        
        let handle = get_runtime_handle();
        let client = reqwest::Client::new();
        let resp = handle.spawn(async move {
            client
                .get(url)
                .header("User-Agent", USER_AGENT)
                .header("Cookie", cookie)
                .send()
                .await
        }).await??;
        
        println!("\n========== API Response (User Info) ==========");
        println!("URL: {}", url);
        println!("Status: {}", resp.status());
        println!("Response Headers:");
        for (key, value) in resp.headers().iter() {
            if let Ok(val_str) = value.to_str() {
                println!("  {}: {}", key, val_str);
            }
        }
        
        let text = resp.text().await?;
        println!("Response Body: {}", text);
        println!("==============================================\n");
        
        #[derive(serde::Deserialize)]
        struct NavPendant { image: Option<String> }
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct NavData { uname: Option<String>, face: Option<String>, pendant: Option<NavPendant> }
        #[derive(serde::Deserialize)]
        struct NavResp { code: i64, data: Option<NavData> }
        let parsed: NavResp = serde_json::from_str(&text).unwrap_or(NavResp{ code: -1, data: None });
        println!("[HomeView] 🔍 解析用户信息响应，code: {}", parsed.code);
        if parsed.code == 0 {
            if let Some(d) = parsed.data {
                println!("[HomeView] 📝 原始头像URL: {:?}", d.face);
                
                // 下载头像到本地
                let face_local = if let Some(face_url) = &d.face {
                    match crate::utils::download_avatar(face_url) {
                        Ok(path_arc) => {
                            // 将 Arc<Path> 转换为字符串用于存储
                            let path_str = path_arc.display().to_string();
                            println!("[HomeView] ✅ 头像下载成功: {}", path_str);
                            Some(path_str)
                        }
                        Err(e) => {
                            println!("[HomeView] ❌ 头像下载失败: {}", e);
                            None
                        }
                    }
                } else {
                    None
                };
                
                // 使用本地缓存路径优先，没有则使用网络URL
                let user = UserProfile { 
                    uname: d.uname.clone(), 
                    face: d.face.clone(),
                    face_local, // 本地缓存路径
                    pendant_image: d.pendant.and_then(|p| p.image) 
                };
                
                println!("[HomeView] ✅ 用户信息构建完成");
                println!("[HomeView]    - 用户名: {:?}", user.uname);
                println!("[HomeView]    - 头像URL: {:?}", user.face);
                println!("[HomeView]    - 本地头像: {:?}", user.face_local);
                println!("[HomeView]    - 挂件图片: {:?}", user.pendant_image);
                
                app_state.update(cx, |s, cx| {
                    s.set_user(user);
                    s.persist_login(); // 保存用户信息到文件
                    cx.notify(); // 触发重新渲染
                })?;
                println!("[HomeView] 🔄 触发UI重新渲染");
            }
        }
        Ok(())
    }
}

impl Render for HomeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 启动登录流程 / 已登录则拉取用户信息
        let (is_logged, started, has_user) = self
            .app_state
            .read_with(cx, |s, _| (s.is_logged_in(), s.qr_started(), s.user().is_some()));
        if !is_logged && !started {
            Self::start_qr_flow(self.app_state.clone(), cx);
        } else if is_logged && !has_user {
            let app_state = self.app_state.clone();
            cx.spawn(async move |_: WeakEntity<HomeView>, cx: &mut AsyncApp| {
                Self::fetch_user_info(app_state, cx).await.ok();
                Ok::<(), anyhow::Error>(())
            })
            .detach();
        }

        let theme = self.app_state.read(cx).theme();
        let bg = match theme { Theme::Dark => rgb(0x000000), Theme::Light => rgb(0xffffff) };
        let fg = match theme { Theme::Dark => rgb(0xffffff), Theme::Light => rgb(0x000000) };

        // 如果已登录，显示欢迎页面
        if is_logged {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .bg(bg)
                .text_color(fg)
                .child(
                    // 胶囊形搜索框 - 完全居中
                    div()
                        .w(px(800.0))
                        .h(px(56.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .px_6()
                        .pr_2()
                        .mt(px(-32.0)) // 向上偏移以补偿 titlebar 高度
                        .rounded_full() // 完全的胶囊形状
                        .bg(match theme {
                            Theme::Dark => rgb(0x1a1a1a), // 更淡的灰色
                            Theme::Light => rgb(0xf8f8f8),
                        })

                        .child(
                            // 自定义输入框UI - 完全自定义的外观
                            {
                                let input_value = self.search_input.read(cx).value().to_string();
                                let is_focused = self.search_input.read(cx).focus_handle(cx).is_focused(_window);
                                let placeholder = if input_value.is_empty() { "搜索你感兴趣的内容..." } else { "" };
                                
                                div()
                                    .flex_1()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .px_4()
                                    .relative() // 必须有relative才能让absolute的子元素正确定位
                                    .cursor(CursorStyle::IBeam)
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(|view, _, window, cx| {
                                        // 点击时聚焦输入框
                                        println!("🖱️  [CustomInput] 点击自定义输入框区域，聚焦输入框");
                                        view.search_input.read(cx).focus_handle(cx).focus(window);
                                        cx.notify(); // 触发重新渲染以显示光标
                                    }))
                                    .child(
                                        // 自定义的文本显示（flex容器，占满空间）
                                        div()
                                            .flex()
                                            .flex_1()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                // 文本内容
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .text_lg()
                                                            .font_weight(FontWeight::NORMAL)
                                                            .text_color(if input_value.is_empty() {
                                                                // 占位符颜色
                                                                match theme {
                                                                    Theme::Dark => rgb(0x666666),
                                                                    Theme::Light => rgb(0x999999),
                                                                }
                                                            } else {
                                                                // 输入文本颜色
                                                                match theme {
                                                                    Theme::Dark => rgb(0xffffff),
                                                                    Theme::Light => rgb(0x333333),
                                                                }
                                                            })
                                                            .child(if input_value.is_empty() { placeholder.to_string() } else { input_value.clone() })
                                                    )
                                                    .when(is_focused, |this| {
                                                        // 光标 - 获得焦点时显示
                                                        this.child(
                                                            div()
                                                                .w(px(2.0))
                                                                .h(px(20.0))
                                                                .bg(match theme {
                                                                    Theme::Dark => rgb(0xffffff),
                                                                    Theme::Light => rgb(0x333333),
                                                                })
                                                                .rounded_sm()
                                                        )
                                                    })
                                            )
                                            .when(!input_value.is_empty(), |this| {
                                                // 清除按钮 - 有内容时显示
                                                this.child(
                                                    div()
                                                        .w(px(20.0))
                                                        .h(px(20.0))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .rounded_full()
                                                        .cursor(CursorStyle::PointingHand)
                                                        .bg(match theme {
                                                            Theme::Dark => rgb(0x333333),
                                                            Theme::Light => rgb(0xcccccc),
                                                        })
                                                        .hover(|this| this.bg(match theme {
                                                            Theme::Dark => rgb(0x444444),
                                                            Theme::Light => rgb(0xbbbbbb),
                                                        }))
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(match theme {
                                                                    Theme::Dark => rgb(0xffffff),
                                                                    Theme::Light => rgb(0x666666),
                                                                })
                                                                .child(IconName::Close)
                                                        )
                                                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|view, _, window, cx| {
                                                            println!("🗑️  [CustomInput] 点击清除按钮");
                                                            view.search_input.update(cx, |state, cx| {
                                                                state.set_value("", window, cx);
                                                            });
                                                            cx.notify();
                                                            cx.stop_propagation(); // 阻止事件冒泡
                                                        }))
                                                )
                                            })
                                    )
                                    // 隐藏的真实Input（只用来处理键盘输入，不显示UI）
                                    .child(
                                        div()
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .w(px(1.0))
                                            .h(px(1.0))
                                            .overflow_hidden()
                                            .child(
                                                input::Input::new(&self.search_input)
                                                    .w(px(1.0))
                                                    .appearance(false)
                                            )
                                    )
                            }
                        )
                        .child(
                            // 搜索按钮 - 在输入框内部，圆形，放大的图标
                            div()
                                .w(px(56.0))
                                .h(px(56.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full() // 圆形按钮
                                .cursor(CursorStyle::PointingHand)
                                .hover(|style| style.bg(match theme {
                                    Theme::Dark => rgb(0x2a2a2a),
                                    Theme::Light => rgb(0xe8e8e8),
                                }))
                                .child(
                                    div()
                                        .text_xl() // 放大图标
                                        .text_color(match theme {
                                            Theme::Dark => rgb(0xaaaaaa),
                                            Theme::Light => rgb(0x666666),
                                        })
                                        .child(IconName::Search)
                                )
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|view, _, _, cx| {
                                    println!("🔍 [SearchButton] 搜索按钮被点击");
                                    let current_value = view.search_input.read(cx).value().to_string();
                                    println!("📝 [SearchButton] 当前输入框的值: '{}'", current_value);
                                    Self::trigger_search(view, cx);
                                    cx.stop_propagation();
                                }))
                        )
                );
        }

        // 未登录，显示扫码登录页面（去除背景，保持居中）
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .text_color(fg)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .p_8()
                    .child(
                        div().text_xl().font_weight(FontWeight::BOLD).child("扫码登录")
                    )
                    .child({
                        if let Some(svg) = self.app_state.read(cx).qr_svg() {
                            let img_arc = Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Svg, svg.to_vec()));
                            img(img_arc)
                                .w(px(240.0))
                                .object_fit(ObjectFit::Contain)
                                .into_any_element()
                        } else {
                            div().w(px(240.0)).h(px(240.0)).into_any_element()
                        }
                    })
                    .child({
                        let status = self.app_state.read_with(cx, |s, _| s.qr_status().to_string());
                        div().text_sm().text_color(match theme { Theme::Dark => rgb(0xaaaaaa), Theme::Light => rgb(0x666666) }).child(status)
                    })
                    .child(
                        button::Button::new("refresh-qr")
                            .outline()
                            .label("刷新二维码")
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.app_state.update(cx, |s, _| { s.set_qr_started(false); s.set_qr_svg(None); s.set_qrcode_key(None); s.set_qr_status("正在获取二维码..."); });
                                cx.notify();
                                Self::request_qr(view.app_state.clone(), cx);
                            }))
                    )
            )
    }
}
