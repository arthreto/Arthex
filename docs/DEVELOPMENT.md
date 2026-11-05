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
  modules/<id>/   one folder per module
src-tauri/
  build.rs        declares the commands granted in capabilities/ + embeds the core locales for Rust
  capabilities/   what the window may call
  src/app/        setup, settings store, overlay window, tray, hotkey, autostart, i18n, logging, storage, commands
  src/services/   system services shared by modules (file index, icons, installed programs, Windows shell, text folding)
  src/modules/<id>/  Rust side of a module: its state, commands and events
  windows/        NSIS installer hooks
```

- **Stack:** [Tauri 2](https://tauri.app) (Rust) with a [Svelte 5](https://svelte.dev) + TypeScript interface, built by Vite.
- **Single window:** it is created hidden at startup and only shown by the hotkey or the tray, which keeps opening instant.

## Where things live

Reuse these instead of writing your own.

| Need                                                                   | Use                                                                                       |
| ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Call Rust                                                              | the typed wrappers: `commands` in `src/core/api.ts` (shell) or `src/modules/<id>/api.ts`  |
| Listen to Rust                                                         | `on(events.x, handler)` from `src/core/api.ts`                                            |
| Show a Rust error                                                      | `errorMessage(error)` from `src/core/api.ts`                                              |
| Read or change settings                                                | `settings.value` and `settings.update(patch)` from `src/core/settings/settings.svelte.ts` |
| Text                                                                   | `t('key')` from `src/core/i18n` (shell) or the module's `i18n.ts`                         |
| A module's name or description                                         | `moduleText(id, 'name' \| 'description')` from `src/core/i18n`                            |
| A button, switch, modal, panel, chip, card, checkbox, file icon, field | `src/ui/`                                                                                 |
| Colours, radii, veil                                                   | the tokens in `src/ui/tokens.css`                                                         |
| Match text against a search                                            | `matchesAll` and `normalize` from `src/core/text.ts` (case and accents ignored)           |
| Show a time ("2 min ago"), a size ("1.2 GB") or a share                | `formatWhen`, `formatBytes` and `formatPercent` from `src/core/format.ts`                 |
| Copy a path                                                            | `copyPath` from `src/core/clipboard.ts`                                                   |
| Feedback after an action                                               | `toast.show(message)` from `src/core/toast.svelte.ts`                                     |
| Open a page, the hub or settings                                       | `navigation.open(view)` from `src/app/navigation.svelte.ts`                               |
| Open the uninstall dialog                                              | `dialogs.uninstall = true` from `src/app/dialogs.svelte.ts`                               |
| Module list, tab bar, launcher order                                   | `modules`, `getModule`, `tabBarIds`, `launcherIds` from `src/core/modules`                |

## Modules

A module is a folder in `src/modules/<id>/`. Its manifest declares the **elements** it provides, and each element has its own switch in Settings → Modules:

| Element  | What it gives                                                   |
| -------- | --------------------------------------------------------------- |
| `page`   | a page in "All pages", and a tab if the user puts it in the bar |
| `widget` | a card on Home                                                  |
| `search` | results in the search bar                                       |
| `tool`   | a mini-app found by name in the search, opened over the overlay |

```
manifest.ts          what the module is (id, icon, tint, order) and the elements it provides, below
<Name>Page.svelte    its page, loaded on first open (pages only)
<Name>Widget.svelte  its card on Home, if any (`widget: { slot: 'top' | 'main' | 'side', component }`)
<Name>Tool.svelte    its tool, if any (`tool`), receiving an optional `input`
<Action>.svelte      a control next to its first switch in Settings → Modules, if any (`settingsAction`)
<Name>Settings.svelte  a section of its own in Settings, if any (`settingsSection: { component, action? }`)
<Name>Pill.svelte    a small status at the top right of the overlay, if any (`badge`)
search.ts            its search results, if any (`search`, see Search below)
locales/<lang>.json  its text: at least "name" and "description", plus "keywords" that help find its tool
i18n.ts              its typed translator
api.ts               typed wrappers of its Rust commands (if any)
*.test.ts            tests of its pure logic
```

- The registry discovers manifests with `import.meta.glob`. **Adding a module means adding a folder**; the shell is not edited.
- `id` is lowercase `[a-z0-9-]`, at most 32 characters. It never changes once released, because settings store it.
- Layout rules are pure functions in `src/core/modules/layout.ts`:
  - `isEnabled` and `switchElement`: the switch of one element;
  - `tabBarIds`: home first, max 6, no disabled pages;
  - `launcherIds`: pinned first, then enabled, then disabled;
  - `toggleFavorite`.
- Switches live in `settings.disabledElements` as `<id>:<element>` (`notes:widget`). The "Turned off" zone of "All pages" drives the `page` switch.
- A turned-off element disappears: the page from the tab bar and "All pages", the card from Home, the results from the search.
- A module is **active** while at least one of its elements is on. Its Settings section, its control in Settings → Modules and its badge show only then, and its Rust service stops when it is not. `sync` in `src-tauri/src/modules/mod.rs` lists the elements of each module that runs in the background.

### Search

A module adds results to the search bar with a `SearchProvider` (`src/core/search/types.ts`):

- `search(query)` returns `SearchResult`s, synchronously or not. `query` carries the typed `text`, a `normalized` copy and `commandsOnly` (the query starts with `>`).
- `order` places its group among the others: calculator 0, tools 5, apps 10, sites 20, system commands 30, files 40.
- `exclusive: true` hides every other group when it answers (the calculator).
- A result names its `primary` action (Enter) and may add a `secondary` one (Ctrl + Enter), a `path` to copy, a `confirm` question for actions that cannot be undone, or a `panel` component that replaces the preview buttons.

The shell asks every enabled provider at each keystroke and drops answers to older queries.

### Tools

A tool is a mini-app with its own interface, such as a colour picker. The shell finds it by its name and `keywords`, in a "Tools" group before the applications, without any search code in the module. Enter opens it over the overlay, in a sheet the size of "What's new"; Escape or a click outside goes back to the search. `tools.open(id, input)` from `src/app/tools.svelte.ts` opens a tool from elsewhere, and the tool receives `input`.

### Create a module

1. Create `src/modules/timer/manifest.ts`:

   ```ts
   import { defineModule } from '../../core/modules/manifest';

   export default defineModule({
     id: 'timer',
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
5. Run `npm run i18n:check`. The page appears in All pages and in Settings → Modules.

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

| Event                     | Payload                 | Meaning                                                           |
| ------------------------- | ----------------------- | ----------------------------------------------------------------- |
| `overlay://shown`         | none                    | the overlay was just shown                                        |
| `settings://changed`      | `Settings`              | settings changed, from any source                                 |
| `app://navigate`          | target id               | open `hub`, `settings` or a page                                  |
| `app://confirm-uninstall` | none                    | the tray asked to uninstall                                       |
| `hotkey://held`           | key names               | modifiers held while recording (Windows)                          |
| `hotkey://recorded`       | key names               | recording finished, `[]` when cancelled (Windows)                 |
| `file-index://updated`    | none                    | a file index scan finished                                        |
| `clipboard://changed`     | none                    | the clipboard history changed (Windows)                           |
| `updates://available`     | `{ version }`           | the check at launch found an update it does not install by itself |
| `updates://progress`      | `{ downloaded, total }` | download progress of an update                                    |

## Data files

Modules keep their data in JSON files of the app data folder, read and written with `storage::read_json` and `storage::write_json` (`src-tauri/src/app/storage.rs`). Writes are atomic, and a corrupt file is set aside instead of crashing.

| File               | Folder     | Content                                                        |
| ------------------ | ---------- | -------------------------------------------------------------- |
| `settings.json`    | config     | settings (see below)                                           |
| `shortcuts.json`   | data       | Home shortcuts                                                 |
| `recent.json`      | data       | recently used programs (path, name, time; never window titles) |
| `notes.json`       | data       | notes (id, title, Markdown text, time)                         |
| `whats-new.json`   | data       | notes of an update being installed, shown once after it        |
| `file-search.json` | config     | folders added to the file index and the exclusions             |
| `index.bin`        | local data | the file index cache, rebuilt from the drives when missing     |

Icons reach the interface as small PNG data URLs made by `services::icons::png_data_url`, so the webview never loads a file or a remote image.

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
- **Clipboard history** (`src-tauri/src/modules/clipboard/`):
  - A message-only window registered with `AddClipboardFormatListener` hears each copy, so nothing is polled.
  - Nothing is read while the module is off or paused. Copies marked `ExcludeClipboardContentFromMonitorProcessing`, `Clipboard Viewer Ignore` or `CanIncludeInClipboardHistory = 0` (password managers) are skipped.
  - Text only, 100 KB at most, 50 entries besides pinned ones, in memory only. Turning the module off forgets everything.
- **File index** (`src-tauri/src/services/file_index/`), shared by file search and FileSize, runs while either is on:
  - It covers the fixed drives (the home folder on other systems) plus the folders the user adds, minus the exclusions.
  - Names live in two shared buffers (as typed and folded), with one compact entry per file pointing to its parent, so a million files take tens of megabytes.
  - The scan runs on background-priority threads, 30 s after a start at login. The cache loads first, so search works at once.
  - Changes reported by `notify` are gathered for a second, then each path is checked on disk once: added, refreshed or hidden, so saves through a temporary file leave no duplicates. A full rescan runs every 6 hours, or at once from Settings. Results whose file is gone are never shown.
  - Excluded folders stay out of search but still count in FileSize, as one entry holding their total size.
  - FileSize works out folder sizes, kinds of content (by place and extension) and the largest files in one pass over the index each time a folder opens (`usage.rs`). Windows, Program Files, profiles, `AppData`, game libraries and what sits directly inside them are protected: they never go to the Recycle Bin, and Rust checks it again before moving anything. Only fixed drives have a Recycle Bin, so items elsewhere are refused; if the shell still cannot recycle an item (larger than the bin), it asks before deleting for good. Open and reveal accept only indexed items on a drive letter.
  - Search splits the query into words that must all match, in parallel with `rayon` and `memchr`. Exact names rank first, then names that start with the query, then the most recently modified.

## Releasing

Once, before the first release, create the update signing key on your own computer, outside the repository (Windows command prompt shown; it asks for a password):

```bash
mkdir %USERPROFILE%\.tauri
```

```bash
npx @tauri-apps/cli signer generate -w %USERPROFILE%\.tauri\arthex.key
```

- Put the content of `arthex.key.pub` in `plugins.updater.pubkey` of `src-tauri/tauri.conf.json`. Until then, builds never look for updates and the release workflow refuses to run.
- In GitHub → Settings → Secrets and variables → Actions, add `TAURI_SIGNING_PRIVATE_KEY` (the content of `arthex.key`) and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- Keep the private key safe and never commit it. Without it, installed copies can no longer be updated.

Then, for each release:

1. Set the same version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
2. Write the changelog (format below) as the message of an annotated tag on `main`, then push the tag:

   ```bash
   git tag -a --cleanup=verbatim vX.Y.Z
   ```

   `--cleanup=verbatim` matters: by default Git drops the lines that start with `#`, so the headings would be lost and the workflow would refuse the tag.

   ```bash
   git push origin vX.Y.Z
   ```

   Or run the Release workflow by hand (Actions → Release → Run workflow, or through the API) with the changelog as its `notes` input: it releases the version of `package.json` and creates the tag itself.

3. The release workflow checks the versions and the changelog, builds the installer, signs it in a separate step (the only one that sees the key; its actions are pinned to commits), then publishes the GitHub Release with `latest.json`.

The **changelog is mandatory**. It becomes the Release description and the `notes` of `latest.json`. Updates are installed by the app (automatically at launch, or from Settings → Updates) only when their signature matches the public key; their notes are kept in `whats-new.json`, and at the next launch the app shows the section of the active language in **What's new**.

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
