use gpui::*;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Mutex;

/// 动画检测状态
#[derive(Clone, Debug)]
enum AnimationStatus {
    Checking,          // 检测中
    Static,            // 静态图片
    Animated(Arc<AnimationData>),  // 动画（已解码）
}

/// 全局动画状态缓存 - 记录每个文件的动画状态
static ANIMATION_STATUS_CACHE: once_cell::sync::Lazy<Mutex<HashMap<String, AnimationStatus>>> = 
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// 全局动画播放状态 - 跟踪每个动画的当前帧
static ANIMATION_PLAY_STATES: once_cell::sync::Lazy<Mutex<HashMap<String, AnimationPlayState>>> = 
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug)]
struct AnimationData {
    frames: Vec<Arc<Image>>,
    frame_delays: Vec<Duration>,
}

#[derive(Clone)]
struct AnimationPlayState {
    current_frame: usize,
    last_update: std::time::Instant,
    warmed_up: bool, // 标记是否已经预热过（快速循环一遍所有帧）
    warmup_frame: usize, // 预热时的帧索引
}

/// 动画头像组件，支持动态webp和gif
pub struct AnimatedAvatar {
    image_path: String, // 保存路径用于查找全局状态
    image_source: AvatarImageSource,
    size: Pixels,
}

enum AvatarImageSource {
    Static(ImageSource),
}

impl AnimatedAvatar {
    /// 创建新的动画头像组件 - 先显示静态，后台异步检测动画
    pub fn new(path_or_url: impl AsRef<str>, size: Pixels) -> Self {
        let source_str = path_or_url.as_ref();
        let image_path = source_str.to_string();
        
        // 先使用静态图片源（快速显示）
        let image_source = if Path::new(source_str).exists() {
            let arc_path: Arc<Path> = Arc::from(Path::new(source_str));
            AvatarImageSource::Static(ImageSource::from(arc_path))
        } else {
            AvatarImageSource::Static(ImageSource::from(source_str.to_string()))
        };
        
        // 检查是否可能是动画文件
        let path_lower = image_path.to_lowercase();
        if (path_lower.ends_with(".webp") || path_lower.ends_with(".gif")) && Path::new(&image_path).exists() {
            // 检查缓存状态
            let mut should_check = false;
            {
                let mut cache = ANIMATION_STATUS_CACHE.lock().unwrap();
                match cache.get(&image_path) {
                    Some(AnimationStatus::Animated(_)) | Some(AnimationStatus::Static) => {
                        // 已知状态，不需要重新检测
                    }
                    Some(AnimationStatus::Checking) => {
                        // 正在检测中，不需要重复启动
                    }
                    None => {
                        // 需要检测
                        cache.insert(image_path.clone(), AnimationStatus::Checking);
                        should_check = true;
                    }
                }
            }
            
            // 如果需要检测，在后台线程异步处理
            if should_check {
                println!("[AnimatedAvatar] 后台检测动画: {}", image_path);
                Self::check_and_decode_async(image_path.clone());
            }
        }
        
        Self {
            image_path,
            image_source,
            size,
        }
    }
    
    /// 在后台线程异步检测和解码动画WebP
    fn check_and_decode_async(path: String) {
        std::thread::spawn(move || {
            // 检测是否为动画WebP
            let is_animated = Self::is_animated_webp_sync(&path);
            
            if is_animated {
                // 解码动画
                if let Some(animation_data) = Self::decode_webp_animation(&path) {
                    // 更新缓存
                    let mut cache = ANIMATION_STATUS_CACHE.lock().unwrap();
                    cache.insert(path.clone(), AnimationStatus::Animated(animation_data.clone()));
                    drop(cache);
                    
                    // 初始化播放状态（预热模式）
                    let mut states = ANIMATION_PLAY_STATES.lock().unwrap();
                    states.entry(path.clone()).or_insert_with(|| AnimationPlayState {
                        current_frame: 0,
                        last_update: std::time::Instant::now(),
                        warmed_up: false,  // 需要预热
                        warmup_frame: 0,   // 从第0帧开始预热
                    });
                    drop(states);
                    
                    println!("[AnimatedAvatar] 后台解码完成: {}, 帧数: {}", path, animation_data.frames.len());
                } else {
                    // 解码失败，标记为静态
                    let mut cache = ANIMATION_STATUS_CACHE.lock().unwrap();
                    cache.insert(path, AnimationStatus::Static);
                }
            } else {
                // 不是动画，标记为静态
                let mut cache = ANIMATION_STATUS_CACHE.lock().unwrap();
                cache.insert(path, AnimationStatus::Static);
            }
        });
    }
    
    /// 同步检测webp是否为动画格式（仅在后台线程调用）
    fn is_animated_webp_sync(path: &str) -> bool {
        use std::fs;
        use webp_animation::Decoder;
        
        // 读取文件
        let data = match fs::read(path) {
            Ok(d) => d,
            Err(_) => return false,
        };
        
        // 尝试作为动画解码
        match Decoder::new(&data) {
            Ok(decoder) => {
                // 计算帧数 - webp-animation的Decoder需要into_iter
                let frame_count = decoder.into_iter().count();
                // 多于1帧才是动画
                frame_count > 1
            }
            Err(_) => false,
        }
    }
    
    /// 解码webp动画 - 返回AnimationData
    fn decode_webp_animation(path: &str) -> Option<Arc<AnimationData>> {
        use std::fs;
        use webp_animation::Decoder;
        
        // 读取文件
        let data = fs::read(path).ok()?;
        
        // 尝试解码为动画
        let decoder = Decoder::new(&data).ok()?;
        
        let mut frames = Vec::new();
        let mut frame_delays = Vec::new();
        
        // 解码所有帧
        let mut last_timestamp = 0;
        for frame in decoder {
            // webp_animation的Frame不是Result，直接使用
            let width = frame.dimensions().0 as u32;
            let height = frame.dimensions().1 as u32;
            let rgba_data = frame.data().to_vec();
            let timestamp = frame.timestamp() as u64;
            
            // 计算帧延迟（当前帧时间戳 - 上一帧时间戳）
            let delay = if last_timestamp == 0 {
                100 // 默认100ms
            } else {
                timestamp.saturating_sub(last_timestamp).max(16) // 最小16ms，限制最高60fps
            };
            last_timestamp = timestamp;
            
            // 创建GPUI Image - 直接从RGBA数据创建
            // 使用image crate创建DynamicImage，然后转换
            if let Some(img_buffer) = image::RgbaImage::from_raw(width, height, rgba_data) {
                let dynamic_img = image::DynamicImage::ImageRgba8(img_buffer);
                
                // 转换为PNG bytes
                let mut png_bytes = Vec::new();
                if dynamic_img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).is_ok() {
                    // 从PNG bytes创建GPUI Image
                    let gpui_image = Image::from_bytes(ImageFormat::Png, png_bytes);
                    frames.push(Arc::new(gpui_image));
                    frame_delays.push(Duration::from_millis(delay));
                }
            }
        }
        
        if frames.is_empty() {
            return None;
        }
        
        Some(Arc::new(AnimationData {
            frames,
            frame_delays,
        }))
    }
    
    /// 设置大小
    #[allow(dead_code)]
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl Render for AnimatedAvatar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 检查是否有动画数据可用
        let cache = ANIMATION_STATUS_CACHE.lock().unwrap();
        let animation_status = cache.get(&self.image_path).cloned();
        drop(cache);
        
        match animation_status {
            Some(AnimationStatus::Animated(animation_data)) => {
                // 动画已就绪，播放动画
                let total_frames = animation_data.frames.len();
                
                // 从全局状态获取当前帧
                let mut states = ANIMATION_PLAY_STATES.lock().unwrap();
                let state = states.get_mut(&self.image_path).unwrap();
                
                // GPU预热阶段：快速循环所有帧一次，让GPUI加载到GPU
                if !state.warmed_up {
                    // 每次render推进预热帧
                    state.warmup_frame += 1;
                    
                    if state.warmup_frame >= total_frames {
                        // 预热完成，开始正常播放
                        state.warmed_up = true;
                        state.current_frame = 0;
                        state.last_update = std::time::Instant::now();
                        println!("[AnimatedAvatar] GPU预热完成: {}, 共{}帧", self.image_path, total_frames);
                    }
                    
                    // 预热期间显示第一帧（用户看不到快速切换）
                    let warmup_frame_to_show = &animation_data.frames[state.warmup_frame.min(total_frames - 1)];
                    drop(states);
                    
                    img(warmup_frame_to_show.clone())
                        .size(self.size)
                        .rounded_full()
                        .into_any_element()
                } else {
                    // 正常播放模式
                    // 检查是否需要切换到下一帧
                    let current_delay = animation_data.frame_delays.get(state.current_frame)
                        .copied()
                        .unwrap_or(Duration::from_millis(100));
                    
                    let elapsed = state.last_update.elapsed();
                    if elapsed >= current_delay {
                        // 切换到下一帧
                        state.current_frame = (state.current_frame + 1) % total_frames;
                        state.last_update = std::time::Instant::now();
                    }
                    
                    let current_frame = &animation_data.frames[state.current_frame];
                    drop(states);
                    
                    // 显示当前帧（刷新由App全局驱动器处理）
                    img(current_frame.clone())
                        .size(self.size)
                        .rounded_full()
                        .into_any_element()
                }
            }
            Some(AnimationStatus::Checking) | None | Some(AnimationStatus::Static) => {
                // 检测中、未知或静态图片，都显示静态图片
                match &self.image_source {
                    AvatarImageSource::Static(source) => {
                        img(source.clone())
                            .size(self.size)
                            .rounded_full()
                            .into_any_element()
                    }
                }
            }
        }
    }
}
