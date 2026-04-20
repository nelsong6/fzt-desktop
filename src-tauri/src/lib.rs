use std::process::Command;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Scaffold-stage: fresh pwsh per call. Follow-up will switch to a
// long-lived warm runspace once fzt-automate's cadence is exercised.
#[tauri::command]
fn run_command(cmd: String) -> Result<String, String> {
    let mut pwsh = Command::new("pwsh");
    pwsh.arg("-NoProfile").arg("-Command").arg(&cmd);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        pwsh.creation_flags(CREATE_NO_WINDOW);
    }

    let out = pwsh.output().map_err(|e| format!("spawn failed: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, run_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
