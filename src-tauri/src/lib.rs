use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// Random 32-hex tail makes accidental collisions with user-command
// output effectively impossible; the warm session reads stdout until it
// sees a line whose trimmed content equals this string exactly.
const SENTINEL: &str = "__fzt_desktop_eot_marker_b9f3ea47c21d4f06__";

/// Long-lived `pwsh -Command -` child. Stdin/stdout pipes stay open for
/// the app lifetime so subsequent run_command invocations pay no PS
/// startup cost and can see each other's variables / $PWD / etc.
struct PwshSession {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl PwshSession {
    fn spawn() -> std::io::Result<Self> {
        let mut cmd = Command::new("pwsh");
        cmd.args(["-NoProfile", "-NoLogo", "-NonInteractive", "-Command", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Drops parse/start-up errors from native commands. *>&1 in
            // run() catches PowerShell-side errors for everything except
            // script-parse failures. Acceptable for the scaffold pass.
            .stderr(Stdio::null());

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().expect("stdin pipe");
        let stdout = BufReader::new(child.stdout.take().expect("stdout pipe"));
        let mut session = Self { _child: child, stdin, stdout };

        // Optional init script — dot-source at spawn so at-commands or
        // other user functions are in scope for every run_command call.
        // Caller sets FZT_DESKTOP_PWSH_INIT to a ps1 path to opt in.
        if let Ok(init) = std::env::var("FZT_DESKTOP_PWSH_INIT") {
            let trimmed = init.trim();
            if !trimmed.is_empty() {
                let escaped = trimmed.replace('\'', "''");
                // Consume output with *>&1; *>$null, then emit sentinel so
                // the drain below knows when init is done.
                writeln!(
                    session.stdin,
                    ". '{escaped}' *>$null; '{SENTINEL}'"
                )
                .ok();
                session.stdin.flush().ok();
                // Drain init output up to the sentinel.
                loop {
                    let mut line = String::new();
                    match session.stdout.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            if line.trim_end_matches(['\r', '\n']) == SENTINEL {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(session)
    }

    fn run(&mut self, cmd: &str) -> Result<String, String> {
        // & { ... } *>&1 merges every PS output stream (success, error,
        // warning, info, verbose, debug) into success so we catch PS-side
        // failures. Trailing sentinel is a bare string literal — emits
        // on the success stream, reaches our pipe reliably.
        writeln!(self.stdin, "& {{ {cmd} }} *>&1; '{SENTINEL}'")
            .map_err(|e| format!("write stdin: {e}"))?;
        self.stdin.flush().map_err(|e| format!("flush stdin: {e}"))?;

        let mut out = String::new();
        loop {
            let mut line = String::new();
            let n = self
                .stdout
                .read_line(&mut line)
                .map_err(|e| format!("read stdout: {e}"))?;
            if n == 0 {
                return Err("pwsh stdout closed mid-command".into());
            }
            if line.trim_end_matches(['\r', '\n']) == SENTINEL {
                break;
            }
            out.push_str(&line);
        }
        Ok(out.trim_end_matches(['\r', '\n']).to_string())
    }
}

// One session for the whole app. Lock contention is a non-issue for
// single-user interactive use; if it ever becomes one, switch to a
// channel-driven worker.
static SESSION: Mutex<Option<PwshSession>> = Mutex::new(None);

/// fzt-automate's menu cache directory. Mirrors the precedence that
/// fzt-automate itself uses (main.go configDir) so both tools read the
/// same file without coordination.
fn fzt_automate_config_dir() -> PathBuf {
    if let Ok(d) = std::env::var("FZT_CONFIG_DIR") {
        if !d.trim().is_empty() {
            return PathBuf::from(d);
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(user) = std::env::var("USERPROFILE") {
            return PathBuf::from(user).join(".fzt-automate");
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("fzt-automate");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".config").join("fzt-automate");
        }
    }
    PathBuf::from(".")
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn run_command(cmd: String) -> Result<String, String> {
    let mut lock = SESSION.lock().map_err(|e| format!("mutex: {e}"))?;
    if lock.is_none() {
        *lock = Some(PwshSession::spawn().map_err(|e| format!("spawn pwsh: {e}"))?);
    }
    lock.as_mut().expect("session just set").run(&cmd)
}

/// Read fzt-automate's menu-cache.yaml so the frontend can mount the
/// real menu. Read-only; fzt-automate keeps ownership of sync + edit.
#[tauri::command]
fn load_menu() -> Result<String, String> {
    let path = fzt_automate_config_dir().join("menu-cache.yaml");
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {}", path.display(), e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, run_command, load_menu])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
