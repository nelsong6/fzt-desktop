// fzt-desktop entry — loads fzt.wasm, mounts the terminal, wires
// keyboard + action routing, and surfaces pwsh output via run_command.
//
// ambience-client.js auto-initializes the pixel-art layer from the
// <canvas data-ambience> element; nothing to wire for that here.

const { invoke } = window.__TAURI__.core;

const FALLBACK_YAML = `- name: "fzt-automate menu not found"
  description: "run fzt-automate once to hydrate ~/.fzt-automate/menu-cache.yaml"
  children:
    - name: "fzt-automate repo"
      description: "where the CLI lives"
      url: https://github.com/nelsong6/fzt-automate
    - name: "this app's readme"
      description: "setup + dev notes"
      url: https://github.com/nelsong6/fzt-desktop
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

async function loadMenu() {
  try {
    return await invoke("load_menu");
  } catch (err) {
    setStatus(`menu-cache unavailable: ${err}`, "err");
    return null;
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

  // Matching fzh-terminal.js's wrapper options. containerPadding:8 leaves
  // breathing room inside the CRT border; defaultCursorPos:null hides the
  // cursor when fzt doesn't emit one rather than placing it at a stub spot.
  const term = createFztWeb(terminalEl, {
    containerPadding: 8,
    defaultCursorPos: null,
    onAction: (action, url) => handleAction(action, url),
  });

  try {
    await term.initWasm();
  } catch (err) {
    setStatus(`fzt.wasm load failed: ${err}`, "err");
    document.getElementById("loading")?.classList.add("hidden");
    return;
  }

  // Register frontend identity (affects the `:` palette injection). No
  // commands yet; we'll add them when fzt-desktop grows its own. Matches
  // fzh-terminal's setFrontend call shape.
  term.setFrontend({ name: "fzt-desktop", version: "dev" });

  const menu = await loadMenu();
  term.loadYAML(menu || FALLBACK_YAML);
  term.init();

  document.getElementById("loading")?.classList.add("hidden");
  if (menu) setStatus("ready");
  terminalEl.focus();
}

window.addEventListener("DOMContentLoaded", init);
