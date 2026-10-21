# Win10 Shell

Windows Phone style Start screen, written in Rust. One codebase for Linux, macOS, Windows, Android, iOS, and a browser PWA.

This is public beta 0.1.0. The Android launcher you already use stays in its own repo. This is the rewrite.

## What it does

- Live clock tile, small / medium / wide tiles, pin, reorder, unpin
- Dark, light, and sunrise-to-sunset themes
- Accent colors and icon shapes
- Settings, diagnostics, and a Buy Me a Coffee link
- In-app update check against [latest.json](latest.json) on this repo

## Platforms

| Platform | How it runs | This beta ships |
| --- | --- | --- |
| Linux x64 | Native window, system WebKitGTK 4.1 | `win10-shell-linux-x64` |
| macOS Apple silicon | Native window, system WebKit | `win10-shell-macos-arm64` |
| Windows x64 | Native window, WebView2 | `win10-shell-windows-x64.exe` |
| PWA | Installable browser app, service worker | `win10-shell-pwa.zip` on the release. Unzip it and serve the folder over https |
| Android | Native shell. The manifest includes `HOME`, so Android can offer it as the launcher | build below |
| iOS | Native shell app. iOS does not allow replacing the system home screen | build below |

Desktop builds use the platform webview, so the binary stays small and uses the GPU the system already has. Release builds use thin LTO, one codegen unit, opt-level 3, stripped symbols, and abort-on-panic.

Linux needs `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, and `libxdo-dev` to compile, and WebKitGTK 4.1 plus `libxdo` to run. Windows needs the WebView2 runtime, which is already on Windows 11. macOS uses the system WebKit. The macOS binary is unsigned: right-click it and choose Open the first time.

### PWA

```
rustup target add wasm32-unknown-unknown
cargo build --release --no-default-features --features web --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/win10-shell.wasm --out-dir dist --target web --out-name win10_shell
cp web/index.html web/manifest.webmanifest web/sw.js web/icon.svg web/icon-192.png web/icon-512.png dist/
```

`wasm-bindgen-cli` must match the `wasm-bindgen` version in `Cargo.lock` (0.2.129 for this beta). Serve `dist/` over https, then install it from the browser. The service worker caches this version and drops the old cache when the cache name changes.

GitHub Pages will also host it at https://wilsonsamiano.github.io/win10-shell/ after Pages is switched on once: [Settings → Pages](https://github.com/wilsonsamiano/win10-shell/settings/pages), source **GitHub Actions**. The release workflow already uploads the site. The token used here cannot flip that switch.

### Android and iOS

The `mobile` feature uses the Dioxus mobile shell (the same renderer as desktop, built for the phone).

```
cargo build --release --no-default-features --features mobile
```

Shipping an APK or IPA also needs the platform SDK, a signing key, and `dx bundle`. This beta does not attach those binaries. [android/AndroidManifest.xml](android/AndroidManifest.xml) adds `HOME` and `DEFAULT` so a bundled Android app can be chosen as the home screen. iOS runs the Start screen as an app. Apple does not allow a third-party app to replace the iPhone home screen.

## Updates

Settings → Check for updates reads `latest.json` from the main branch (a raw GitHub URL, so the browser allows it). When the version there is newer, the app opens the GitHub release. Install over the app you have. The PWA updates when the page is refreshed and the service worker takes the new cache.

## Layout

Tiles and theme are saved in the app data directory on desktop and mobile (`win10-shell/state.json`) and in `localStorage` in the PWA.

https://buymeacoffee.com/wilsonsamiano
