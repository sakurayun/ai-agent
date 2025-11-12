use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, de::DeserializeOwned};

pub fn load_json<T: DeserializeOwned>(path: &str) -> anyhow::Result<Option<T>> {
    if !Path::new(path).exists() { return Ok(None); }
    let data = fs::read_to_string(path)?;
    let v = serde_json::from_str::<T>(&data)?;
    Ok(Some(v))
}

pub fn save_json<T: Serialize>(path: &str, value: &T) -> anyhow::Result<()> {
    let s = serde_json::to_string_pretty(value)?;
    fs::write(path, s)?;
    Ok(())
}

/// 下载网络图片到本地缓存目录，返回绝对路径（使用Arc<Path>格式）
pub fn download_avatar(url: &str) -> anyhow::Result<std::sync::Arc<Path>> {
    download_image(url, "avatar_cache", "头像")
}

/// 下载视频封面到本地缓存目录，返回绝对路径（使用Arc<Path>格式）
pub fn download_cover(url: &str) -> anyhow::Result<std::sync::Arc<Path>> {
    download_image(url, "cover_cache", "封面")
}

/// 通用图片下载函数
fn download_image(url: &str, cache_dir_name: &str, image_type: &str) -> anyhow::Result<std::sync::Arc<Path>> {
    println!("[Utils] 📥 开始下载{}: {}", image_type, url);
    
    // 创建缓存目录
    let cache_dir = PathBuf::from(cache_dir_name);
    fs::create_dir_all(&cache_dir)?;
    let cache_dir_abs = cache_dir.canonicalize()?;
    println!("[Utils] 📁 缓存目录: {:?}", cache_dir_abs);
    
    // 使用URL的hash作为文件名
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    
    // 获取文件扩展名
    let ext = url.split('.').last().unwrap_or("jpg");
    let filename = format!("{}.{}", &hash[..16], ext);
    let file_path = cache_dir.join(&filename);
    
    // 如果文件不存在，下载图片
    if !file_path.exists() {
        println!("[Utils] ⬇️ 正在下载{}...", image_type);
        
        // 添加User-Agent和Referer头，避免CORS问题
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()?;
        
        let response = client.get(url)
            .header("Referer", "https://www.bilibili.com/")
            .send()?;
        
        let bytes = response.bytes()?;
        println!("[Utils] 💾 下载完成，大小: {} bytes", bytes.len());
        fs::write(&file_path, bytes)?;
    } else {
        println!("[Utils] ✅ {}已存在缓存", image_type);
    }
    
    // 获取绝对路径并转换为Arc<Path>
    let abs_path = file_path.canonicalize()?;
    
    println!("[Utils] ✅ 返回路径: {:?}", abs_path);
    println!("[Utils] 📊 文件信息:");
    if let Ok(metadata) = fs::metadata(&abs_path) {
        println!("[Utils]    - 大小: {} bytes", metadata.len());
        println!("[Utils]    - 是否存在: {}", abs_path.exists());
    }
    
    Ok(std::sync::Arc::from(abs_path.as_path()))
}
