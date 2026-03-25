# AGENTS.md

## Project Overview

- Name: `paste`
- Type: desktop application
- Stack: `Tauri 2 + Vue 3 + TypeScript + Vite + Rust`
- Status: scaffold/template stage, not yet business-feature complete

## Goals

- Build a Paste-style clipboard manager on top of the current Tauri + Vue foundation
- Keep frontend and Rust responsibilities clearly separated
- Prefer small, verifiable iterations
- Ship a Windows-first local MVP before expanding scope

## Repository Structure

- `src/`: Vue frontend
- `src/main.ts`: frontend entry
- `src/App.vue`: current main UI entry
- `public/`: static assets
- `src-tauri/`: Tauri and Rust backend
- `src-tauri/src/main.rs`: Rust binary entry
- `src-tauri/src/lib.rs`: Tauri app builder and commands
- `src-tauri/tauri.conf.json`: app/window/build config
- `src-tauri/capabilities/`: Tauri permissions/capabilities

## Current State

- Frontend now renders a clipboard history panel with search, keyboard navigation, pin/delete actions, pause, and clear-history controls
- Rust backend now manages clipboard capture, SQLite persistence, tray integration, and global hotkey handling
- Clipboard history currently supports `text` and `image`
- The app is designed as a tray-resident utility with `Ctrl+Shift+V` as the default global hotkey
- Data is stored locally in SQLite plus an app-data image cache directory
- `@tauri-apps/plugin-opener` is installed and enabled
- `node_modules/` and `src-tauri/target/` already exist locally
- This folder is currently not a Git repository

## Common Commands

- Install deps: `pnpm install`
- Start frontend dev server: `pnpm dev`
- Start Tauri app: `pnpm tauri dev`
- Build frontend: `pnpm build`
- Build/package through Tauri: `pnpm tauri build`

## Collaboration Rules

- Make focused changes instead of broad rewrites
- Preserve user changes unless explicitly asked to replace them
- Update this file when architecture, workflows, or key decisions change
- Document any new Rust command exposed to the frontend
- Document any new Tauri permission/capability before relying on it

## Frontend Notes

- Use Vue 3 SFCs with `<script setup lang="ts">`
- Keep UI logic in `src/` and avoid leaking platform concerns into presentation code
- When app complexity grows, split `App.vue` into feature components early
- The main panel is keyboard-first: search, arrow navigation, `Enter` to copy, `Esc` to hide
- Frontend listens for backend events to refresh clipboard history and app state

## Rust/Tauri Notes

- Put Tauri commands in `src-tauri/src/lib.rs` unless a module split becomes necessary
- Keep command input/output serializable and explicit
- Add only the minimum required capabilities for each feature
- Current module split:
- `app_state.rs`: managed runtime state
- `storage.rs`: SQLite and image persistence
- `clipboard_monitor.rs`: clipboard watching and ingestion
- `hotkey.rs`: global hotkey registration
- `tray.rs`: tray menu and window lifecycle
- `windowing.rs`: show/hide panel helpers
- Core commands exposed to the frontend:
- `get_history`
- `get_app_state`
- `copy_item_to_clipboard`
- `toggle_item_pin`
- `delete_item`
- `clear_history`
- `set_monitoring_paused`
- `hide_panel`

## Working Agreement

- Before major changes, clarify the target outcome if it is ambiguous
- After meaningful changes, verify with a relevant build or smoke test when feasible
- Prefer simple architecture first, then extract abstractions when the product shape is clearer

## Open Items

- Define the product scope for `paste`
- Decide local data storage strategy
- Decide whether the app will be single-window or multi-window
- Decide whether clipboard, file system, or global shortcut capabilities are needed
