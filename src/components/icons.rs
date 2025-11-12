use gpui::*;

// 使用内置的符号字符，无需外部字体
pub mod fa {
    // 导航图标 - 使用更通用的 Unicode 符号
    pub const HOUSE: &str = "⌂"; // House symbol
    pub const GEAR: &str = "⚙"; // Gear symbol (已经在使用)

    // 窗口控制图标 - 使用标准的 Windows 符号
    pub const MINUS: &str = "─"; // Box drawing horizontal line
    pub const WINDOW_MAXIMIZE: &str = "☐"; // Ballot box
    pub const XMARK: &str = "✕"; // Multiplication X
    
    // 其他图标
    #[allow(dead_code)]
    pub const ARROW_RIGHT_FROM_BRACKET: &str = "⇥"; // 退出登录图标
    pub const USER_CIRCLE: &str = "👤"; // 用户图标
    #[allow(dead_code)]
    pub const CHEVRON_UP: &str = "⌃"; // 向上箭头
}

// 专门的导航图标组件，带有默认样式
pub fn nav_icon(icon: &'static str) -> impl IntoElement {
    div()
        .font_family("Segoe UI Symbol")
        .text_size(px(20.0))
        .flex()
        .items_center()
        .justify_center()
        .child(icon)
}

// 专门的窗口控制按钮图标组件
pub fn window_control_icon(icon: &'static str) -> impl IntoElement {
    div()
        .font_family("Segoe UI Symbol")
        .text_size(px(16.0))
        .flex()
        .items_center()
        .justify_center()
        .child(icon)
}
