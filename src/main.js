// fzt-desktop entry — loads fzt.wasm, mounts the terminal, wires
// keyboard + action routing, and surfaces pwsh output via run_command.
//
// ambience-client.js auto-initializes the pixel-art layer from the
// <canvas data-ambience> element; nothing to wire for that here.

const { invoke } = window.__TAURI__.core;

// Placeholder tree shown until fzt-automate's real menu source lands.
// Items with a `url` open the default browser; items with `action:
// <cmd>` route through the hidden pwsh runspace for silent execution.
const SAMPLE_YAML = `- name: fzt-desktop
  description: scaffold — replace this tree with fzt-automate's menu
  children:
    - name: echo hello
      description: pwsh smoke test via run_command
      action: Write-Output hello
    - name: get-date
      description: pwsh smoke test via run_command
      action: Get-Date
    - name: Tauri docs
      description: opens in default browser
      url: https://tauri.app
    - name: fzt-browser
      description: source of this WASM
      url: https://github.com/nelsong6/fzt-browser
`;

function setStatus(text, kind = "") {
  const el = document.getElementById("status");
  if (!el) return;
  el.textContent = text;
  el.className = "status-line" + (kind ? " " + kind : "");
}

async function handleAction(action, url) {
  if (!action || !action.startsWith("select:")) return;

  if (url) {
    const opener = window.__TAURI__?.opener;
    try {
      if (opener?.openUrl) await opener.openUrl(url);
      else window.open(url, "_blank");
    } catch (err) {
      setStatus(`open failed: ${err}`, "err");
    }
    return;
  }

  const cmd = action.slice("select:".length).trim();
  if (!cmd) return;

  setStatus(`running: ${cmd}`);
  try {
    const stdout = await invoke("run_command", { cmd });
    setStatus(stdout.trim() || `done: ${cmd}`, "ok");
  } catch (stderr) {
    setStatus(`err: ${String(stderr).trim()}`, "err");
  }
}

async function init() {
  const terminalEl = document.getElementById("terminal");
  if (!terminalEl) return;

  let createFztWeb;
  try {
    ({ createFztWeb } = await import("./fzt-web.js"));
  } catch (_err) {
    setStatus(
      "fzt-web.js missing — run 'npm run fetch-fzt' in the repo root",
      "err",
    );
    document.getElementById("loading")?.classList.add("hidden");
    return;
  }

  const term = createFztWeb(terminalEl, {
    onAction: (action, url) => handleAction(action, url),
  });

  try {
    await term.initWasm();
  } catch (err) {
    setStatus(`fzt.wasm load failed: ${err}`, "err");
    document.getElementById("loading")?.classList.add("hidden");
    return;
  }

  term.loadYAML(SAMPLE_YAML);
  term.init();

  document.getElementById("loading")?.classList.add("hidden");
  setStatus("ready");
  terminalEl.focus();
}

window.addEventListener("DOMContentLoaded", init);
