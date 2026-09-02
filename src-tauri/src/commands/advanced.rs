use super::gaming::ps_quiet;

#[derive(serde::Serialize, Clone)]
pub struct AdvTool {
    pub id: String,
    pub label: String,
    pub desc: String,
    pub admin: bool,    // cần quyền admin
    pub risky: bool,    // có thể gây gián đoạn (explorer restart, hibernate off)
}

#[tauri::command]
pub fn get_advanced_tools() -> Vec<AdvTool> {
    vec![
        AdvTool { id: "restart_explorer".into(), label: "Khởi động lại Explorer".into(), desc: "Kill + chạy lại explorer.exe — sửa taskbar/start menu bị treo, áp dụng thay đổi registry UI ngay lập tức.".into(), admin: false, risky: true },
        AdvTool { id: "flush_dns".into(), label: "Xóa bộ nhớ cache DNS".into(), desc: "ipconfig /flushdns — xóa cache DNS cục bộ, cần dùng khi đổi DNS hoặc trang web resolve sai.".into(), admin: false, risky: false },
        AdvTool { id: "time_sync".into(), label: "Đồng bộ giờ hệ thống".into(), desc: "w32tm /resync — ép đồng bộ giờ với time server ngay, sửa lệch giờ gây lỗi chứng chỉ/đăng nhập.".into(), admin: false, risky: false },
        AdvTool { id: "clear_thumbcache".into(), label: "Xóa cache thumbnail".into(), desc: "Xóa thumbcache_*.db của Explorer — giải phóng vài trăm MB, ép thumbnail render lại (sửa ảnh hiển thị sai).".into(), admin: false, risky: false },
        AdvTool { id: "hibernate_off".into(), label: "Tắt Hibernate".into(), desc: "powercfg /hibernate off — xóa file hiberfil.sys (thường 3-6GB), tắt chế độ ngủ đông. Không ảnh hưởng Sleep.".into(), admin: true, risky: true },
        AdvTool { id: "hibernate_on".into(), label: "Bật Hibernate".into(), desc: "powercfg /hibernate on — bật lại chế độ ngủ đông, tạo lại hiberfil.sys.".into(), admin: true, risky: false },
        AdvTool { id: "restore_point".into(), label: "Tạo Restore Point".into(), desc: "Checkpoint-Computer — tạo điểm khôi phục hệ thống trước khi thay đổi registry/system files.".into(), admin: true, risky: false },
        AdvTool { id: "sfc_scan".into(), label: "Quét SFC".into(), desc: "sfc /scannow — quét và sửa file hệ thống Windows bị hỏng. Có thể mất 5-15 phút.".into(), admin: true, risky: false },
    ]
}

/// Chạy tool. Trả (ok, output).
#[tauri::command]
pub fn run_advanced_tool(id: String) -> Result<(bool, String), String> {
    let script: &str = match id.as_str() {
        "restart_explorer" => r#"Stop-Process -Name explorer -Force -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 500; Start-Process explorer; 'Explorer đã khởi động lại'"#,
        "flush_dns" => r#"ipconfig /flushdns | Out-String"#,
        "time_sync" => r#"w32tm /resync /force 2>&1 | Out-String"#,
        "clear_thumbcache" => r#"
$files = Get-ChildItem "$env:LOCALAPPDATA\Microsoft\Windows\Explorer\thumbcache_*.db" -ErrorAction SilentlyContinue
if (-not $files) { 'Không có thumbnail cache' } else {
  $total = ($files | Measure-Object Length -Sum).Sum
  $files | Remove-Item -Force -ErrorAction SilentlyContinue
  'Đã xóa {0:N0} file ({1:N1} MB)' -f $files.Count, ($total / 1MB)
}"#,
        "hibernate_off" => r#"powercfg /hibernate off 2>&1 | Out-String; 'Hibernate đã tắt'"#,
        "hibernate_on" => r#"powercfg /hibernate on 2>&1 | Out-String; 'Hibernate đã bật'"#,
        "restore_point" => r#"
try {
  Checkpoint-Computer -Description 'WinOptimizer manual' -RestorePointType MODIFY_SETTINGS -ErrorAction Stop 2>&1 | Out-String
  'Đã tạo restore point'
} catch { 'Lỗi: ' + $_.Exception.Message }"#,
        "sfc_scan" => r#"sfc /scannow 2>&1 | Out-String"#,
        _ => return Err(format!("Tool không tồn tại: {}", id)),
    };
    let out = ps_quiet(script, if id == "sfc_scan" { 900 } else { 60 });
    Ok((!out.to_lowercase().contains("access denied") && !out.contains("Lỗi:"), out))
}