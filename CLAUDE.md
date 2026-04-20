# fzt-desktop

Native Windows desktop app hosting fzt-automate and the ambience ecosystem's pixel-art surface. Tauri 2 shell wrapping a web frontend that reuses fzt-showcase's renderer patterns (DOS font, CRT effects, phosphor glow).

## Why a separate repo

The 2026-04-19 "pixels chat" tried to embed pixel art into fzt-automate's TUI surface via sixel escape sequences. The experiment failed because tcell + sixel + wt.exe don't cohabit: tcell's cell redraws wipe sixel pixels, and wt.exe rendered sixel rasters opaquely regardless of the `Pb=1` transparency flag. Tracked as [ambience#11/#12/#15](https://github.com/nelsong6/ambience/issues).

Nelson's pivotal framing (2026-04-20T04:06Z):

> it's really just the medium fighting us, a typical terminal. we have ways to make our own windows.

fzt-desktop is that own-window. By rendering into a surface we fully control (Tauri's WebView2), the text and pixel layers compose cleanly — no escape-code gymnastics, no cell-grid conflict.

## Architecture

```
Tauri 2 shell (Rust, src-tauri/)
    |
    +-- WebView2 window (Windows-native, Edge-based)
    |    |
    |    +-- Frontend (vanilla JS, src/)
    |         +-- fzt-automate UI (fzt.wasm from fzt-browser releases)
    |         +-- Pixel art canvas (ambience entropy consumer)
    |         +-- CRT aesthetic (ported from fzt-showcase)
    |
    +-- Hidden pwsh runspace (long-lived child process)
         Executes silent automation; interactive commands spawn wt.exe
```

## Execution model

- **Silent commands** (`lights off`, automation, at-command side-effects): hidden pwsh subprocess with warm runspace. Output shown in fzt-automate's status area. No window popups.
- **Interactive commands** (`vim`, `git log`, `ssh`): flagged in the menu via `InteractiveOutput: true` on the ItemAction; spawn `wt.exe <cmd>` for a real terminal.
- **Future** (milestone deferred): integrated terminal pane via xterm.js + ConPTY for the "fzt-automate is the OS" endgame.

## Stack choices

- **Tauri 2** over Electron: ~10MB vs ~150MB shipped; WebView2 is bundled with Windows so no Chromium bundled; DX niceties (hot reload, bundler, signing).
- **Vanilla JS** over React/Svelte: consistency with fzt-showcase and fzt-browser; no framework surface to maintain.
- **Web-tech renderer** over native GDI: reuses fzt-showcase's proven CRT aesthetic (scanlines, vignette, phosphor glow, DOS font) as CSS rather than rebuilding in GDI/D2D. fzt-picker's GDI pattern remains the baseline for constrained-renderer apps; fzt-desktop exists because we wanted the web aesthetic to leverage.

## Dependencies

- **[Tauri 2](https://tauri.app)** — Rust shell + WebView2 integration
- **[fzt-browser](https://github.com/nelsong6/fzt-browser)** — consumed as release assets (`fzt.wasm` + `fzt-terminal.js` + CSS); not a Go module dep here
- **[fzt-showcase](https://github.com/nelsong6/fzt-showcase)** — reference for CRT/DOS/phosphor CSS (to be ported in)
- **[ambience](https://github.com/nelsong6/ambience)** — SSE entropy source; fzt-desktop subscribes and optionally publishes local keystroke events back

## Status

Scaffold only. Default Tauri 2 vanilla-JS template. No fzt integration, no pwsh runspace, no pixel art, no ambience subscription yet.

## Related session

Full design thread in the "pixels" chat (Claude Code session `2dff8c57-b6b9-4d4c-a3a5-19429e293c11`, 2026-04-18 — 2026-04-20). Captures the sixel failure, the medium-vs-message reframe, and the Tauri/native renderer trade-off.
