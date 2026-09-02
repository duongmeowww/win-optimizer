use crate::commands::gaming::ps_quiet;

/// Thực hiện dọn dẹp bộ nhớ tạm thời (Temp, Prefetch) và trả về (ok, message).
#[tauri::command]
pub async fn run_memory_cleaner() -> Result<(bool, String), String> {
    // PowerShell script to clean %TEMP% and prefetch files.
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
# Clean user Temp folder
$temp = $env:TEMP
if (Test-Path $temp) {
  Get-ChildItem -Path $temp -Recurse -Force -ErrorAction SilentlyContinue | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
}
# Clean Windows Prefetch (requires admin for system folder)
$prefetch = "$env:SystemRoot\System32\Prefetch"
if (Test-Path $prefetch) {
  Get-ChildItem -Path $prefetch -Recurse -Force -ErrorAction SilentlyContinue | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
}
'Cleanup hoàn thành.'
"#;
    let out = ps_quiet(script, 30);
    let ok = !out.to_lowercase().contains("access denied");
    Ok((ok, out))
}
