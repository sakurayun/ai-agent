use std::sync::Arc;
use gpui::*;
use gpui_component::*;
use crate::state::app_state::{AppState, Theme, Cookies, UserProfile};
use qrcode::QrCode;
use qrcode::render::svg;

// Bilibili API 的标准 User-Agent
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

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
            let response = client
                .get(url)
                .header("User-Agent", USER_AGENT)
                .send()
                .await?;
            
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
                
                let response = client
                    .get(&url)
                    .header("User-Agent", USER_AGENT)
                    .send()
                    .await?;
                
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
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
                let Some(data) = parsed.data else {
                    tokio::time::sleep(Duration::from_secs(2)).await;
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
                        
                        let response = client
                            .get(refresh_url)
                            .header("User-Agent", USER_AGENT)
                            .send()
                            .await?;
                        
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
                tokio::time::sleep(Duration::from_secs(2)).await;
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
        
        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Cookie", cookie)
            .send()
            .await?;
        
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
        struct NavData { uname: Option<String>, face: Option<String>, pendant: Option<NavPendant> }
        #[derive(serde::Deserialize)]
        struct NavResp { code: i64, data: Option<NavData> }
        let parsed: NavResp = serde_json::from_str(&text).unwrap_or(NavResp{ code: -1, data: None });
        if parsed.code == 0 {
            if let Some(d) = parsed.data {
                let user = UserProfile { uname: d.uname, face: d.face, pendant_image: d.pendant.and_then(|p| p.image) };
                app_state.update(cx, |s, _| s.set_user(user))?;
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
                                            .child(
                                                img(user.face.clone().unwrap_or_default())
                                                    .w(px(72.0))
                                                    .h(px(72.0))
                                                    .rounded_full()
                                                    .object_fit(ObjectFit::Cover)
                                            )
                                            .child({
                                                if let Some(p) = &user.pendant_image {
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
                                            .child(user.uname.clone().unwrap_or_else(|| "用户".to_string()))
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
