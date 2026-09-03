use crate::commands::debloat::apply_debloat;
use crate::commands::gaming::apply_gaming_tweak;
use crate::commands::memory_cleaner::run_memory_cleaner;

/// Thực hiện tối ưu hóa một cú click: dọn dẹp RAM, gỡ bloatware an toàn, áp dụng tweaks game an toàn.
#[tauri::command]
pub async fn one_click_optimize() -> Result<(bool, String), String> {
    let mut success = true;
    let mut messages = Vec::new();

    // 1. Dọn dẹp bộ nhớ (async)
    match run_memory_cleaner().await {
        Ok((ok, msg)) => {
            if ok {
                messages.push(format!("✅ Memory Cleaner: {}", msg));
            } else {
                success = false;
                messages.push(format!("❌ Memory Cleaner: {}", msg));
            }
        }
        Err(e) => {
            success = false;
            messages.push(format!("❌ Memory Cleaner error: {}", e));
        }
    }

    // 2. Gỡ bloatware an toàn (không cần admin) - synchronous
    let safe_debloats = [
        "appx_spotify",
        "appx_tiktok",
        "appx_clipchamp",
        "appx_teams",
        "appx_news",
        "appx_maps",
        "appx_candycrush",
        "appx_solitaire",
    ];
    for id in safe_debloats.iter() {
        // apply_debloat is synchronous, not async
        match apply_debloat(id.to_string(), "apply".to_string()) {
            Ok((ok, msg)) => {
                if ok {
                    messages.push(format!("✅ Debloat {}: {}", id, msg));
                } else {
                    success = false;
                    messages.push(format!("❌ Debloat {}: {}", id, msg));
                }
            }
            Err(e) => {
                success = false;
                messages.push(format!("❌ Debloat {} error: {}", id, e));
            }
        }
    }

    // 3. Áp dụng tweaks game an toàn (chỉ những tweak thuộc HKCU, không cần admin)
    let safe_gaming = [
        ("game_mode", "apply"),
        ("game_dvr", "apply"),
        ("mouse", "apply"),
    ];
    for (id, action) in safe_gaming.iter() {
        match apply_gaming_tweak(id.to_string(), action.to_string()).await {
            Ok((ok, msg)) => {
                if ok {
                    messages.push(format!("✅ Gaming {}: {}", id, msg));
                } else {
                    success = false;
                    messages.push(format!("❌ Gaming {}: {}", id, msg));
                }
            }
            Err(e) => {
                success = false;
                messages.push(format!("❌ Gaming {} error: {}", id, e));
            }
        }
    }

    let summary = messages.join("\n");
    Ok((success, summary))
}