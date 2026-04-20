use std::io::{BufRead, BufReader, Write};
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
        cmd.args(["-NoProfile", "-NoLogo", "-Command", "-"])
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
        Ok(Self { _child: child, stdin, stdout })
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, run_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
