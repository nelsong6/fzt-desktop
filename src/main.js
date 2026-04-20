// fzt-desktop frontend — Tauri 2 + vanilla JS.
//
// This is a scaffold. Real integrations land in subsequent commits:
//   - fzt.wasm loading (from fzt-browser release, bundled by CI)
//   - Ambience SSE subscription + keystroke publish-back
//   - Hidden pwsh runspace via Tauri `invoke('run_command', ...)`
//
// For now this just boots the canvas so the visual surface is obvious
// when `tauri dev` is run.

const { invoke } = window.__TAURI__.core;

// ── Ambience canvas (pixel-art layer) ─────────────────────────────
// Placeholder for the entropy-driven pixel art that was the originating
// use case. Will subscribe to ambience.romaine.life SSE once wired.
function initAmbienceCanvas() {
  const canvas = document.getElementById("ambience-canvas");
  if (!canvas) return;

  const resize = () => {
    canvas.width = window.innerWidth * devicePixelRatio;
    canvas.height = window.innerHeight * devicePixelRatio;
  };
  resize();
  window.addEventListener("resize", resize);

  // Marks the body so fzt-terminal background goes transparent and the
  // canvas shows through — matches fzt-showcase's ambience-on pattern.
  document.body.classList.add("ambience-on");

  // Future: const source = new EventSource("https://ambience.romaine.life/events");
  // Future: render entropy-driven pixel art to the canvas here
}

// ── Status line ───────────────────────────────────────────────────
// Small feedback surface for silent command execution. When fzt-automate
// selects an at-command, the Rust side runs it in the hidden pwsh
// runspace and the result lands here.
function setStatus(text, kind = "") {
  const el = document.getElementById("status");
  if (!el) return;
  el.textContent = text;
  el.className = "status-line" + (kind ? " " + kind : "");
}

// ── Boot ───────────────────────────────────────────────────────────
window.addEventListener("DOMContentLoaded", () => {
  initAmbienceCanvas();
  setStatus("scaffold ready — fzt.wasm not yet wired");

  // Keep the greet invoke around as a live Rust-side test until we have
  // real Tauri commands. Safe to delete once `run_command` lands.
  if (typeof invoke === "function") {
    invoke("greet", { name: "fzt-desktop" })
      .then((msg) => console.log("[rust]", msg))
      .catch((err) => console.warn("[rust] greet failed:", err));
  }
});
