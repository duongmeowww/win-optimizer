use serde_json::Value;

#[derive(serde::Serialize, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_url: String,
    pub release_notes: String,
}

const REPO_OWNER: &str = "duongmeowww";
const REPO_NAME: &str = "win-optimizer";
const CURRENT_VERSION: &str = "0.4.0";

#[tauri::command]
pub async fn check_for_update() -> Result<UpdateInfo, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let url = format!("https://api.github.com/repos/{}/{}/releases/latest", REPO_OWNER, REPO_NAME);
        let client = reqwest::blocking::Client::builder()
            .user_agent("WinOptimizer-App")
            .build()
            .map_err(|e| e.to_string())?;

        let resp = match client.get(&url).send() {
            Ok(r) => r,
            Err(_) => {
                return Ok(UpdateInfo {
                    current_version: CURRENT_VERSION.to_string(),
                    latest_version: CURRENT_VERSION.to_string(),
                    has_update: false,
                    release_url: format!("https://github.com/{}/{}/releases", REPO_OWNER, REPO_NAME),
                    release_notes: String::new(),
                });
            }
        };

        if !resp.status().is_success() {
            return Ok(UpdateInfo {
                current_version: CURRENT_VERSION.to_string(),
                latest_version: CURRENT_VERSION.to_string(),
                has_update: false,
                release_url: format!("https://github.com/{}/{}/releases", REPO_OWNER, REPO_NAME),
                release_notes: String::new(),
            });
        }

        let json: Value = resp.json().map_err(|e| e.to_string())?;
        let latest_tag = json["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
        let body = json["body"].as_str().unwrap_or("").to_string();
        let html_url = json["html_url"].as_str().unwrap_or("").to_string();

        let has_update = parse_version(&latest_tag) > parse_version(CURRENT_VERSION);
        Ok(UpdateInfo {
            current_version: CURRENT_VERSION.to_string(),
            latest_version: if latest_tag.is_empty() { CURRENT_VERSION.to_string() } else { latest_tag },
            has_update,
            release_url: if html_url.is_empty() { format!("https://github.com/{}/{}/releases", REPO_OWNER, REPO_NAME) } else { html_url },
            release_notes: body,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn parse_version(v: &str) -> (u32, u32, u32) {
    let parts: Vec<&str> = v.split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}
