use anyhow::Context as _;
use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "*.png"]
#[include = "*.svg"]
#[include = "*.ico"]
#[include = "*.ttf"]
#[include = "icons/**/*.svg"]
#[include = "fonts/**/*.ttf"]
#[exclude = "*.DS_Store"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        // 支持同时传入 "logo.png" 或 "assets/logo.png" 形式的路径
        let normalized = path
            .trim_start_matches('/')
            .trim_start_matches(".\\")
            .trim_start_matches("./");
        let key = if let Some(stripped) = normalized.strip_prefix("assets/") {
            stripped
        } else if let Some(stripped) = normalized.strip_prefix("assets\\") {
            stripped
        } else {
            normalized
        };

        Self::get(key)
            .map(|f| Some(f.data))
            .with_context(|| format!("loading asset at path {path:?}"))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| {
                if p.starts_with(path) {
                    Some(p.into())
                } else {
                    None
                }
            })
            .collect())
    }
}
