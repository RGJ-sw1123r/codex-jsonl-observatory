# Codex JSONL Observatory

A local-first Rust + Svelte web viewer for Codex CLI JSONL session logs.

## Runtime

The target runtime is a Tauri desktop app. The Svelte frontend remains the app UI,
and the Tauri shell calls the Rust parser/API boundary directly for local JSONL
path parsing.

Development and verification entry points:

```text
cd frontend
npm run check
npm run build
npm run tauri:dev
npm run tauri:build
```

Release zips should be assembled from Tauri build outputs, including the Windows
`.exe` produced under `frontend/src-tauri/target/release/` or the generated
bundle artifact under `frontend/src-tauri/target/release/bundle/`.
