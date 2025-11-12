use serde::{Deserialize, Serialize};
use anyhow::Result;

/// 合集元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonMeta {
    pub season_id: i64,
    pub name: String,
    pub total: i32,
    pub description: Option<String>,
}

/// 系列元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesMeta {
    pub series_id: i64,
    pub name: String,
    pub total: i32,
    pub description: Option<String>,
}

/// 合集数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub meta: SeasonMeta,
}

/// 系列数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub meta: SeriesMeta,
}

/// 合集和系列列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsLists {
    pub seasons_list: Option<Vec<Season>>,
    pub series_list: Option<Vec<Series>>,
}

/// 用户空间合集列表响应数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceCollectionsData {
    pub items_lists: ItemsLists,
}

/// API 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: Option<String>,
    pub data: Option<T>,
}

/// 视频简要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoArchive {
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub pic: String,
    pub pubdate: i64,
    pub duration: i64,
    pub stat: VideoStat,
}

/// 视频统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStat {
    pub view: i64,
    #[serde(default)]
    pub vt: i64,
    #[serde(default)]
    pub like: Option<i64>,
    #[serde(default)]
    pub coin: Option<i64>,
    #[serde(default)]
    pub favorite: Option<i64>,
    #[serde(default)]
    pub reply: Option<i64>,
    #[serde(default)]
    pub share: Option<i64>,
    #[serde(default)]
    pub danmaku: Option<i64>,
}

/// 合集视频列表响应数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonArchivesData {
    pub archives: Vec<VideoArchive>,
    pub page: PageInfo,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub page_num: i32,
    pub page_size: i32,
    pub total: i32,
}

/// 获取用户空间的合集和视频系列列表
pub async fn fetch_space_collections(
    mid: &str,
    cookie: &str,
    page_num: i32,
    page_size: i32,
) -> Result<SpaceCollectionsData> {
    let url = "https://api.bilibili.com/x/polymer/web-space/seasons_series_list";
    
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .query(&[
            ("mid", mid),
            ("page_num", &page_num.to_string()),
            ("page_size", &page_size.to_string()),
            ("web_location", "333.1387"),
        ])
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", cookie)
        .header("Referer", "https://space.bilibili.com/")
        .send()
        .await?;
    
    let api_response: ApiResponse<SpaceCollectionsData> = response.json().await?;
    
    if api_response.code != 0 {
        anyhow::bail!("API 返回错误: code={}, message={:?}", api_response.code, api_response.message);
    }
    
    api_response.data.ok_or_else(|| anyhow::anyhow!("API 返回数据为空"))
}

/// 获取合集的视频列表（单页）
pub async fn fetch_season_archives(
    mid: &str,
    season_id: &str,
    cookie: &str,
    page_num: i32,
    page_size: i32,
) -> Result<SeasonArchivesData> {
    let url = "https://api.bilibili.com/x/polymer/web-space/seasons_archives_list";
    
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .query(&[
            ("mid", mid),
            ("season_id", season_id),
            ("sort_reverse", "false"),
            ("page_num", &page_num.to_string()),
            ("page_size", &page_size.to_string()),
            ("web_location", "333.1387"),
        ])
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", cookie)
        .header("Referer", "https://space.bilibili.com/")
        .send()
        .await?;
    
    // 解析 JSON 响应
    let api_response: ApiResponse<SeasonArchivesData> = response.json().await?;
    
    if api_response.code != 0 {
        anyhow::bail!("API 返回错误: code={}, message={:?}", api_response.code, api_response.message);
    }
    
    api_response.data.ok_or_else(|| anyhow::anyhow!("API 返回数据为空"))
}

/// 获取合集的所有视频列表（自动翻页）
pub async fn fetch_all_season_archives(
    mid: &str,
    season_id: &str,
    cookie: &str,
) -> Result<Vec<VideoArchive>> {
    let mut all_videos = Vec::new();
    let mut page_num = 1;
    let page_size = 30;
    
    loop {
        let data = fetch_season_archives(mid, season_id, cookie, page_num, page_size).await?;
        
        if data.archives.is_empty() {
            break;
        }
        
        let total = data.page.total;
        all_videos.extend(data.archives);
        
        if all_videos.len() >= total as usize {
            break;
        }
        
        page_num += 1;
    }
    
    Ok(all_videos)
}
