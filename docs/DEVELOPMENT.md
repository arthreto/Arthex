# Developing Arthex

How Arthex is built and how to extend it: modules, Rust commands, settings, languages and releases. The project rules (workflow, interface and code conventions) are in [`CLAUDE.md`](../CLAUDE.md).

## Build from source

Requirements:

- [Node.js](https://nodejs.org) 22 or later.
- [Rust](https://rustup.rs) stable.
- The [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your system: WebView2 and the MSVC build tools on Windows, WebKitGTK on Linux.

```bash
npm install
```

```bash
npm run tauri dev
```

```bash
npm run tauri build
```

`npm run tauri build` writes the installer to `src-tauri/target/release/bundle/`. Logs are written daily to the app log folder. `ARTHEX_LOG=debug` raises their level.

## Checks

Run them before every commit. CI runs the same checks and builds the Windows installer.

```bash
npm run check && npm run lint && npm test && npm run i18n:check
```

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

`npm run format` fixes formatting of the interface code.

## Project structure

```
src/
  app/            shell: App, Overlay, SearchBar, TabBar, Hub, KeyHints, settings/
  core/           api.ts (typed commands and events), i18n/, settings/, modules/ (registry), appearance, toast
  ui/             shared components + tokens.css (themes) + base.css
  modules/<id>/   one folder per page or feature
src-tauri/
  build.rs        declares the commands granted in capabilities/ + embeds the core locales for Rust
  capabilities/   what the window may call
  src/app/        setup, settings store, overlay window, tray, autostart, i18n, logging, commands
  windows/        NSIS installer hooks
```

- **Stack:** [Tauri 2](https://tauri.app) (Rust) with a [Svelte 5](https://svelte.dev) + TypeScript interface, built by Vite.
- **Single window:** it is created hidden at startup and only shown by the hotkey or the tray, which keeps opening instant.

## Where things live

Reuse these instead of writing your own.

| Need                                  | Use                                                                                       |
| ------------------------------------- | ----------------------------------------------------------------------------------------- |
| Call Rust                             | the typed wrappers: `commands` in `src/core/api.ts` (shell) or `src/modules/<id>/api.ts`  |
| Listen to Rust                        | `on(events.x, handler)` from `src/core/api.ts`                                            |
| Show a Rust error                     | `errorMessage(error)` from `src/core/api.ts`                                              |
| Read or change settings               | `settings.value` and `settings.update(patch)` from `src/core/settings/settings.svelte.ts` |
| Text                                  | `t('key')` from `src/core/i18n` (shell) or the module's `i18n.ts`                         |
| A module's name or description        | `moduleText(id, 'name' \| 'description')` from `src/core/i18n`                            |
| A button, switch, modal, keycap, icon | `src/ui/`                                                                                 |
| Colours, radii, veil                  | the tokens in `src/ui/tokens.css`                                                         |
| Feedback after an action              | `toast.show(message)` from `src/core/toast.svelte.ts`                                     |
| Open a page, the hub or settings      | `navigation.open(view)` from `src/app/navigation.svelte.ts`                               |
| Module list, tab bar, launcher order  | `modules`, `getModule`, `tabBarIds`, `launcherIds` from `src/core/modules`                |

## Modules

A page (Home, FileSize, Notes…) or a feature without a page (calculator, system commands…) is a folder in `src/modules/<id>/`:

```
manifest.ts          what the module is
<Name>Page.svelte    its page, loaded on first open (pages only)
locales/<lang>.json  its text: at least "name" and "description"
i18n.ts              its typed translator
api.ts               typed wrappers of its Rust commands (if any)
*.test.ts            tests of its pure logic
```

- The registry discovers manifests with `import.meta.glob`. **Adding a module means adding a folder**; the shell is not edited.
- `id` is lowercase `[a-z0-9-]`, at most 32 characters. It never changes once released, because settings store it.
- Layout rules are pure functions in `src/core/modules/layout.ts`:
  - `tabBarIds`: home first, max 6, no disabled modules;
  - `launcherIds`: pinned first, then enabled, then disabled;
  - `toggleFavorite`.
- A disabled module disappears everywhere: tab bar, search results, home widgets. Its Rust service stops.

### Create a module

1. Create `src/modules/timer/manifest.ts`:

   ```ts
   import { defineModule } from '../../core/modules/manifest';

   export default defineModule({
     id: 'timer',
     kind: 'page',
     icon: 'M12 7v5l3 2M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z', // 24×24 stroked SVG path
     tint: 'green', // one of the --tint-* tokens
     order: 50,
     page: () => import('./TimerPage.svelte'),
   });
   ```

2. Add `locales/en.json`, then one file per other language:

   ```json
   { "name": "Timer", "description": "Countdowns and a Pomodoro.", "start": "Start" }
   ```

3. Add `i18n.ts`:

   ```ts
   import { moduleTranslator } from '../../core/i18n';
   import type en from './locales/en.json';

   export const t = moduleTranslator<typeof en>('timer');
   ```

4. Write `TimerPage.svelte` with components from `src/ui/` and text from `t('start')`.
5. Run `npm run i18n:check`. The module appears in All pages.

## Rust commands and events

The interface talks to Rust only through typed commands. The window may only call what `src-tauri/capabilities/main.json` grants.

To add a command:

1. Write it as a `#[tauri::command]` function and validate every input.
   - Shell commands go in `src-tauri/src/app/commands.rs`.
   - Module commands go in `src-tauri/src/modules/<id>/commands.rs`, named `<id>_<action>` (e.g. `notes_save`).
   - Return `app::Result<T>`. Errors reach the interface as `{ kind, message }`.
2. Grant it: add `allow-<name-with-dashes>` to `src-tauri/capabilities/main.json`. `build.rs` reads the capabilities, so this line also declares the command.
3. Register it in `generate_handler!` in `src-tauri/src/lib.rs`.
4. Add a typed wrapper.
   - Shell commands get it in `commands` in `src/core/api.ts`.
   - Module commands get it in `src/modules/<id>/api.ts`.
   - Components never call `invoke` directly.

Events go from Rust to the interface.

- Shell events are declared in `src-tauri/src/app/events.rs` and in `events` in `src/core/api.ts`.
- Module events are named `<id>://<event>`.

| Event                     | Payload    | Meaning                                           |
| ------------------------- | ---------- | ------------------------------------------------- |
| `overlay://shown`         | none       | the overlay was just shown                        |
| `settings://changed`      | `Settings` | settings changed, from any source                 |
| `app://navigate`          | target id  | open `hub`, `settings` or a page                  |
| `app://confirm-uninstall` | none       | the tray asked to uninstall                       |
| `hotkey://held`           | key names  | modifiers held while recording (Windows)          |
| `hotkey://recorded`       | key names  | recording finished, `[]` when cancelled (Windows) |

## Settings

- Rust owns them: `settings.json` in the app config folder, versioned and written atomically. A corrupt file is set aside and replaced by the defaults.
- The interface sends **partial patches**, for example `settings.update({ theme: 'dark' })`.
  - Rust rejects unknown keys and normalises the result. For example, Home stays first in the tab bar and a hotkey has 1 to 4 keys.
  - Rust then emits `settings://changed`, so every view and the tray menu stay in sync.
- To add a setting, change three places together:
  - the `Settings` struct in `src-tauri/src/app/settings.rs`, with its default and normalisation;
  - `src/core/settings/types.ts`;
  - a test in `settings.rs`.
- A breaking change bumps `VERSION` in `settings.rs` and migrates old files.

## Languages

- Messages use the ICU format (`intl-messageformat`). It handles variables (`{name}`), plurals and select.
  - Shell text lives in `src/core/locales/<lang>.json`, which also carries `$meta` (code, name, native name).
  - Module text lives in `src/modules/<id>/locales/<lang>.json`, under the `modules.<id>.*` namespace.
- English is the reference and the fallback. Keys are typed from `en.json`, so an unknown key does not compile.
- Rust translates the tray menu (`tray.*`) from the same `core/locales` files, embedded at build time.
- Dates, sizes and numbers go through `Intl` in the active language.

### Add a language

```bash
npm run i18n:new de
```

This creates every `de.json` (shell and modules) from English. Translate them, then check that nothing is missing:

```bash
npm run i18n:check
```

The language appears in Settings and in the tray at the next build. No code change is needed.

## Platform notes

- **Shift + Win** cannot be registered with `RegisterHotKey`, because it is made only of modifiers.
  - It uses a `WH_KEYBOARD_LL` hook on its own thread, driven by a pure, tested state machine.
  - It fires when Shift + Win is released with no other key in between, so Win + Shift + S still works.
  - A neutral `vkE8` key press is injected so that releasing Win does not open the Start menu.
  - The hook callback does O(1) work. Windows silently removes slow hooks.
- The startup entry is managed by `tauri-plugin-autostart`. It launches the app with `--autostart`, so it starts silently. Debug builds never register it.
- Install is NSIS per user. `src-tauri/windows/hooks.nsh` removes the startup entry on uninstall.
- Data lives in the Tauri app folders named after the identifier `dev.arthreto.arthex`. The uninstaller offers to delete them.

## Releasing

1. Set the same version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
2. Tag `vX.Y.Z` on `main` and push the tag.
3. The release workflow builds and signs the installers, then publishes the GitHub Release with `latest.json`.

The **Release description is the changelog and is mandatory**. It becomes the `notes` of `latest.json`. At the first launch after an update, the app shows the section of the active language in **What's new**.

It holds two sections, in this order, with the same items:

```
### English
#### New
- …
#### Improved
- …
#### Fixed
- …

### Français
#### Nouveau
- …
#### Amélioré
- …
#### Corrigé
- …
```

- Omit an empty subsection.
- Write for users, not developers: what changed for them, one short line each.
