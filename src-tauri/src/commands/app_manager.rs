use crate::commands::gaming::ps_quiet;
use tauri::async_runtime::spawn_blocking;
use std::time::Duration;

/// Một ứng dụng phổ biến có thể cài đặt qua Winget
#[derive(serde::Serialize, Clone)]
pub struct PopularApp {
    pub id: String,           // Winget ID
    pub name: String,         // Tên hiển thị
    pub publisher: String,    // Nhà phát triển
    pub version: String,      // Phiên bản hiện tại
    pub category: String,     // Danh mục: Browser, Office, Development, Utility
    pub installed: bool,      // Đã cài đặt chưa?
}

/// Trả về danh sách ứng dụng phổ biến có thể cài đặt qua Winget.
#[tauri::command]
pub async fn get_popular_apps() -> Vec<PopularApp> {
    spawn_blocking(move || {
        // Danh sách ứng dụng phổ biến được hard-coded để tránh phụ thuộc vào winget search lúc runtime
        // Trong tương lai có thể thay bằng winget search --output-format json
        let apps = vec![
            PopularApp {
                id: "Microsoft.Edge".to_string(),
                name: "Microsoft Edge".to_string(),
                publisher: "Microsoft".to_string(),
                version: "".to_string(),
                category: "Browser".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Google.Chrome".to_string(),
                name: "Google Chrome".to_string(),
                publisher: "Google".to_string(),
                version: "".to_string(),
                category: "Browser".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Mozilla.Firefox".to_string(),
                name: "Mozilla Firefox".to_string(),
                publisher: "Mozilla".to_string(),
                version: "".to_string(),
                category: "Browser".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Microsoft.VisualStudioCode".to_string(),
                name: "Visual Studio Code".to_string(),
                publisher: "Microsoft".to_string(),
                version: "".to_string(),
                category: "Development".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Git.Git".to_string(),
                name: "Git".to_string(),
                publisher: "GitSCM".to_string(),
                version: "".to_string(),
                category: "Development".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Microsoft.PowerToys".to_string(),
                name: "PowerToys".to_string(),
                publisher: "Microsoft".to_string(),
                version: "".to_string(),
                category: "Utility".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Discord.Discord".to_string(),
                name: "Discord".to_string(),
                publisher: "Discord Inc.".to_string(),
                version: "".to_string(),
                category: "Utility".to_string(),
                installed: false,
            },
            PopularApp {
                id: "OBS.Project".to_string(),
                name: "OBS Studio".to_string(),
                publisher: "OBS Project".to_string(),
                version: "".to_string(),
                category: "Utility".to_string(),
                installed: false,
            },
            PopularApp {
                id: "7zip.7zip".to_string(),
                name: "7-Zip".to_string(),
                publisher: "7zip".to_string(),
                version: "".to_string(),
                category: "Utility".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Notepad++.Notepad++".to_string(),
                name: "Notepad++".to_string(),
                publisher: "Notepad++ Team".to_string(),
                version: "".to_string(),
                category: "Utility".to_string(),
                installed: false,
            },
            PopularApp {
                id: "VLC.VLC".to_string(),
                name: "VLC media player".to_string(),
                publisher: "VideoLAN".to_string(),
                version: "".to_string(),
                category: "Utility".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Adobe.Acrobat.Reader.DC".to_string(),
                name: "Adobe Acrobat Reader DC".to_string(),
                publisher: "Adobe".to_string(),
                version: "".to_string(),
                category: "Office".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Microsoft.Teams".to_string(),
                name: "Microsoft Teams".to_string(),
                publisher: "Microsoft".to_string(),
                version: "".to_string(),
                category: "Office".to_string(),
                installed: false,
            },
            PopularApp {
                id: "SlackTechnologies.Slack".to_string(),
                name: "Slack".to_string(),
                publisher: "Slack Technologies".to_string(),
                version: "".to_string(),
                category: "Office".to_string(),
                installed: false,
            },
            PopularApp {
                id: "Zoom.Zoom".to_string(),
                name: "Zoom".to_string(),
                publisher: "Zoom Video Communications".to_string(),
                version: "".to_string(),
                category: "Office".to_string(),
                installed: false,
            },
        ];

        // Kiểm tra trạng thái cài đặt cho mỗi ứng dụng
        apps.into_iter().map(|mut app| {
            // Kiểm tra qua Winget list hoặc các phương pháp khác
            // Đối với mục đích này, chúng ta sẽ sử dụng một cách tiếp cận đơn giản
            // Trong thực tế, có thể kiểm tra qua registry, file system, hoặc winget list
            app.installed = check_if_installed(&app.id);
            app
        }).collect()
    })
    .await
    .unwrap_or_default()
}

/// Kiểm tra xem một ứng dụng có được cài đặt qua Winget không
fn check_if_installed(id: &str) -> bool {
    // Sử dụng winget list --id <id> --output-format json và kiểm tra kết quả
    // Đối với mục đích này, chúng ta sẽ sử dụng một cách tiếp cận đơn giản hơn
    // Trong thực tế, nên gọi winget và parse output
    let out = ps_quiet(&format!("winget list --id \"{}\" --output-format json", id), 5);
    !out.trim().is_empty() && !out.contains("No installed package found")
}

/// Kiểm tra xem một ứng dụng có sẵn để cài đặt qua Winget không
#[tauri::command]
pub async fn is_available_for_install(id: String) -> bool {
    spawn_blocking(move || {
        let out = ps_quiet(&format!("winget search --id \"{}\" --output-format json", id), 5);
        !out.trim().is_empty() && !out.contains("No packages found")
    })
    .await
    .unwrap_or_default()
}

/// Cài đặt một ứng dụng qua Winget
#[tauri::command]
pub async fn install_app(id: String) -> Result<(bool, String), String> {
    spawn_blocking(move || {
        // Kiểm tra quyền admin
        let is_admin = ps_quiet("(New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)", 5);
        if is_admin.trim() != "True" {
            return Err("Cần chạy ứng dụng với quyền Administrator để cài đặt phần mềm".to_string());
        }

        // Chạy winget install
        let script = format!(r#"winget install --id "{}" --accept-source-agreements --accept-package-agreements --silent --accept-agreements"#, id);
        let out = ps_quiet(&script, 120); // Timeout 2 phút để cài đặt

        // Kiểm tra kết quả
        if out.contains("Successfully installed") || out.trim().ends_with("Success") {
            Ok((true, format!("Đã cài đặt thành công {}", id)))
        } else if out.contains("already installed") {
            Ok((true, format!("{} đã được cài đặt trước đó", id)))
        } else {
            Err(format!("Cài đặt thất bại: {}", out))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Gỡ cài đặt một ứng dụng qua Winget
#[tauri::command]
pub async fn uninstall_app(id: String) -> Result<(bool, String), String> {
    spawn_blocking(move || {
        // Kiểm tra quyền admin
        let is_admin = ps_quiet("(New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)", 5);
        if is_admin.trim() != "True" {
            return Err("Cần chạy ứng dụng với quyền Administrator để gỡ cài đặt phần mềm".to_string());
        }

        // Chạy winget uninstall
        let script = format!(r#"winget uninstall --id "{}" --silent --accept-agreements"#, id);
        let out = ps_quiet(&script, 120); // Timeout 2 phút

        // Kiểm tra kết quả
        if out.contains("Successfully uninstalled") || out.trim().ends_with("Success") {
            Ok((true, format!("Đã gỡ cài đặt thành công {}", id)))
        } else if out.contains("not installed") {
            Ok((true, format!("{} chưa được cài đặt", id)))
        } else {
            Err(format!("Gỡ cài đặt thất bại: {}", out))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}