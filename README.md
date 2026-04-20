# fzt-desktop

Native Windows desktop surface hosting [fzt-automate](https://github.com/nelsong6/fzt-automate) + ambience pixel-art, built on [Tauri 2](https://tauri.app) with WebView2. Full architecture and decision thread in [CLAUDE.md](./CLAUDE.md).

## Develop

```sh
npm install
npm run tauri dev
```

First `tauri dev` compiles Tauri's Rust deps — expect a few minutes on a cold build.

## Build

```sh
npm run tauri build
```

Produces an NSIS installer and MSI under `src-tauri/target/release/bundle/`. CI mirrors this and uploads to the release.

## Related

- [fzt-browser](https://github.com/nelsong6/fzt-browser) — source of `fzt.wasm` (downloaded at build time)
- [fzt-showcase](https://github.com/nelsong6/fzt-showcase) — reference for the CRT/DOS aesthetic
- [ambience](https://github.com/nelsong6/ambience) — entropy source for the pixel-art layer
