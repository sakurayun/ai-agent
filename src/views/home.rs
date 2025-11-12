use std::sync::{Arc, OnceLock};
use gpui::*;
use gpui_component::*;
use crate::state::app_state::{AppState, Theme, Cookies, UserProfile};
use crate::components::AnimatedAvatar;
use qrcode::QrCode;
use qrcode::render::svg;

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
}

impl HomeView {
    pub fn new(app_state: Entity<AppState>, _window: &Window, _cx: &mut Context<Self>) -> Self {
        Self { app_state }
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
                .items_center()
                .justify_center()
                .bg(bg)
                .text_color(fg)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_6()
                        .p_8()
                        .child(
                            div()
                                .text_2xl()
                                .font_weight(FontWeight::BOLD)
                                .child("欢迎回来！")
                        )
                        .child({
                            if let Some(user) = self.app_state.read(cx).user() {
                                // 克隆用户数据以避免借用检查问题
                                let avatar_path = if let Some(local_path) = &user.face_local {

                                    local_path.clone()
                                } else if let Some(face_url) = &user.face {
                                    println!("[HomeView] 🌐 使用远程URL: {}", face_url);
                                    face_url.clone()
                                } else {
                                    println!("[HomeView] ⚠️ 没有头像数据");
                                    String::new()
                                };
                                let pendant_image = user.pendant_image.clone();
                                let uname = user.uname.clone();
                                
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .gap_4()
                                    .child(
                                        div()
                                            .relative()
                                            .w(px(88.0))
                                            .h(px(88.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child({
                                                // 使用AnimatedAvatar组件支持动画webp
                                                cx.new(|_| AnimatedAvatar::new(avatar_path, px(72.0)))
                                            })
                                            .child({
                                                if let Some(p) = pendant_image {
                                                    img(p.clone())
                                                        .absolute()
                                                        .top(px(-8.0))
                                                        .left(px(-8.0))
                                                        .w(px(88.0))
                                                        .h(px(88.0))
                                                        .object_fit(ObjectFit::Contain)
                                                        .into_any_element()
                                                } else {
                                                    div().into_any_element()
                                                }
                                            })
                                    )
                                    .child(
                                        div()
                                            .text_xl()
                                            .child(uname.unwrap_or_else(|| "用户".to_string()))
                                    )
                                    .into_any_element()
                            } else {
                                div()
                                    .text_color(match theme { Theme::Dark => rgb(0xaaaaaa), Theme::Light => rgb(0x666666) })
                                    .child("正在加载用户信息...")
                                    .into_any_element()
                            }
                        })
                        .child(
                            button::Button::new("logout")
                                .outline()
                                .label("退出登录")
                                .on_click(cx.listener(|view, _, _, cx| {
                                    view.app_state.update(cx, |s, _| {
                                        s.set_logged_in(false);
                                        s.set_cookies(Cookies::default());
                                        s.set_user(UserProfile {
                                            uname: None,
                                            face: None,
                                            face_local: None,
                                            pendant_image: None,
                                        });
                                        s.set_qr_started(false);
                                        s.set_qr_svg(None);
                                        s.set_qrcode_key(None);
                                        s.persist_login();
                                    });
                                    cx.notify();
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
