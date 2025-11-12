use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Page {
    Home,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

pub struct AppState {
    current_page: Page,
    theme: Theme,
    logged_in: bool,
    cookies: Option<Cookies>,
    user: Option<UserProfile>,
    // 登录页所需的临时状态
    qr_started: bool,
    qrcode_key: Option<String>,
    qr_svg: Option<Vec<u8>>, // SVG bytes
    qr_status: String,
    // UI状态
    user_menu_open: bool,
}

impl AppState {
    pub fn new() -> Self {
        // 尝试从磁盘加载登录状态
        let mut s = Self {
            current_page: Page::Home,
            theme: Theme::Dark,  // 默认使用深色主题
            logged_in: false,
            cookies: None,
            user: None,
            qr_started: false,
            qrcode_key: None,
            qr_svg: None,
            qr_status: String::new(),
            user_menu_open: false,
        };
        if let Ok(Some(saved)) = crate::utils::load_json::<SavedLogin>("bili_cookies.json") {
            s.cookies = Some(saved.cookies.clone());
            s.logged_in = saved.logged_in;
            s.user = saved.user.clone();
        }
        s
    }

    pub fn current_page(&self) -> Page {
        self.current_page
    }

    pub fn set_page(&mut self, page: Page) {
        self.current_page = page;
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
    }

    pub fn is_logged_in(&self) -> bool { self.logged_in }
    pub fn user(&self) -> Option<&UserProfile> { self.user.as_ref() }

    pub fn set_logged_in(&mut self, logged: bool) { self.logged_in = logged; }
    pub fn set_cookies(&mut self, cookies: Cookies) { self.cookies = Some(cookies); }
    pub fn set_user(&mut self, user: UserProfile) { self.user = Some(user); }

    pub fn cookie_header(&self) -> Option<String> {
        let c = self.cookies.as_ref()?;
        let mut parts = vec![];
        if let Some(v) = &c.DedeUserID { parts.push(format!("DedeUserID={}", v)); }
        if let Some(v) = &c.DedeUserID__ckMd5 { parts.push(format!("DedeUserID__ckMd5={}", v)); }
        if let Some(v) = &c.bili_jct { parts.push(format!("bili_jct={}", v)); }
        if let Some(v) = &c.sid { parts.push(format!("sid={}", v)); }
        parts.push(format!("SESSDATA={}", c.SESSDATA));
        Some(parts.join("; "))
    }

    pub fn persist_login(&self) {
        let saved = SavedLogin { 
            logged_in: self.logged_in, 
            cookies: self.cookies.clone().unwrap_or_default(),
            user: self.user.clone(),
        };
        let _ = crate::utils::save_json("bili_cookies.json", &saved);
    }

    // 登录页状态
    pub fn qr_started(&self) -> bool { self.qr_started }
    pub fn set_qr_started(&mut self, v: bool) { self.qr_started = v; }
    pub fn qrcode_key(&self) -> Option<&String> { self.qrcode_key.as_ref() }
    pub fn set_qrcode_key(&mut self, k: Option<String>) { self.qrcode_key = k; }
    pub fn qr_svg(&self) -> Option<&[u8]> { self.qr_svg.as_deref() }
    pub fn set_qr_svg(&mut self, data: Option<Vec<u8>>) { self.qr_svg = data; }
    pub fn qr_status(&self) -> &str { &self.qr_status }
    pub fn set_qr_status(&mut self, s: impl Into<String>) { self.qr_status = s.into(); }
    
    // UI状态
    pub fn is_user_menu_open(&self) -> bool { self.user_menu_open }
    pub fn set_user_menu_open(&mut self, open: bool) { self.user_menu_open = open; }
    pub fn toggle_user_menu(&mut self) { self.user_menu_open = !self.user_menu_open; }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Cookies {
    pub SESSDATA: String,
    pub DedeUserID: Option<String>,
    pub DedeUserID__ckMd5: Option<String>,
    pub bili_jct: Option<String>,
    pub sid: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SavedLogin {
    pub logged_in: bool,
    pub cookies: Cookies,
    pub user: Option<UserProfile>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserProfile {
    pub uname: Option<String>,
    pub face: Option<String>,
    pub face_local: Option<String>, // 本地缓存的头像路径
    pub pendant_image: Option<String>,
}
