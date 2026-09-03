use serde_json::Value;
use std::process::Command;
use std::os::windows::process::CommandExt;

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
const CURRENT_VERSION: &str = "0.3.0";

#[tauri::command]
pub async fn check_for_update() -> Result<UpdateInfo, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let url = format!("https://api.github.com/repos/{}/{}/releases/latest", REPO_OWNER, REPO_NAME);
        
        // Sửa lệnh PowerShell chuẩn: pipe trực tiếp Invoke-RestMethod sang ConvertTo-Json
        let ps_cmd = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $response = Invoke-RestMethod -Uri '{}' -UserAgent 'WinOptimizer-App' -TimeoutSec 10; $response | ConvertTo-Json -Depth 5",
            url
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .map_err(|e| format!("Không thể chạy PowerShell: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Lỗi kết nối GitHub API qua PowerShell: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("Lỗi phân tích JSON từ GitHub: {} (raw: {})", e, &stdout[..std::cmp::min(150, stdout.len())]))?;

        let latest_tag = json["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
        let body = json["body"].as_str().unwrap_or("").to_string();
        let html_url = json["html_url"].as_str().unwrap_or("").to_string();

        if latest_tag.is_empty() {
            return Err(format!("Không tìm thấy tag_name trong GitHub release response. Raw JSON: {}", &stdout[..std::cmp::min(200, stdout.len())]));
        }

        let has_update = parse_version(&latest_tag) > parse_version(CURRENT_VERSION);
        Ok(UpdateInfo {
            current_version: CURRENT_VERSION.to_string(),
            latest_version: latest_tag,
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
