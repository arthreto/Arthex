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

`npm run tauri build` writes the installer to `src-tauri/target/release/bundle/`. Logs are written daily to the app log folder (`%LOCALAPPDATA%\dev.arthreto.arthex\logs` on Windows), which Settings → General opens. The "Debug log" setting (`debugLog`, off by default) adds the detail of what Arthex does, such as when and why the overlay opens and closes: write such traces with `tracing::debug!`. `ARTHEX_LOG` (`debug`, `trace`…) fixes the level and overrides the setting.

## Checks

Run them before every commit. CI runs the same checks and builds the Windows installer.

```bash
npm run check && npm run lint && npm test && npm run i18n:check
```

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

```bash
cd sdk/native && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

`npm run format` fixes formatting of the interface code.

## Project structure

```
src/
  app/            shell: App, Overlay, SearchBar, TabBar, Hub, KeyHints, settings/
  core/           api.ts (typed commands and events), i18n/, settings/, modules/ (registry), appearance, toast
  ui/             shared components + tokens.css (themes) + base.css
  modules/<id>/   one folder per module
sdk/              the kit for installable modules: runtime, build preset, `arthex` command, native/ (Rust crate)
src-tauri/
  build.rs        declares the commands granted in capabilities/ + embeds the core locales for Rust
  capabilities/   what the window may call
  src/app/        setup, settings store, overlay window, tray, hotkey, autostart, i18n, logging, storage, commands
  src/services/   system services shared by modules (file index, icons, installed programs, Windows shell, text folding, module host)
  src/modules/<id>/  Rust side of a module: its state, commands and events
  windows/        NSIS installer hooks
```

- **Stack:** [Tauri 2](https://tauri.app) (Rust) with a [Svelte 5](https://svelte.dev) + TypeScript interface, built by Vite.
- **Single window:** it is created hidden at startup and only shown by the hotkey or the tray, which keeps opening instant.
  - On Windows, `see_through` in `overlay.rs` extends the DWM frame over the whole borderless window at each opening. Without it, Windows fills what the page leaves transparent with black. Windows 11 then puts its acrylic behind the page (`DWMSBT_TRANSIENTWINDOW`): the plain blur alone leaves the window black there. Older versions, which lack it, get the plain blur. The acrylic has its own dark tint, so `--veil` stays light enough to let the desktop show. macOS uses the `hudWindow` effect of the config.
  - The `translucent` setting turns all of this off: `see_through` removes the frame and the backdrop, and `applyAppearance` sets `data-backdrop="solid"`, which replaces `--veil` with the opaque `--veil-solid`.
  - The webview must keep a fully transparent background: `main.rs` sets `WEBVIEW2_DEFAULT_BACKGROUND_COLOR=0` before WebView2 starts, and each opening checks the controller's `DefaultBackgroundColor` and clears it if needed. With the debug log, each opening writes that alpha, the WebView2 version and the graphics the page renders with (software rendering cannot be see-through).
  - The overlay covers the work area of its screen, but never a whole screen (`fitted`): when the taskbar hides itself, it stops one pixel short. Windows shows a topmost window covering a whole screen like a full-screen game, straight to the display, and the transparent page then turns black.
  - Rust hides it with `overlay::hide`, which first sends `overlay://hiding`. The page clears itself, then calls `overlay_hide`. `WebView2` shows its last frame at the next opening, so that frame is empty and the enter animation plays from the start.
  - A focus loss closes the overlay only once the foreground window is really another one: focus moving into the page reports one too.
  - While hidden, release builds suspend `WebView2`. Debug builds do not: suspending cuts the socket of the Vite dev server, which then reloads the page at every opening. Judge how the overlay opens on a release build (`npm run tauri build -- --no-bundle`, then `src-tauri/target/release/arthex.exe`).

## Where things live

Reuse these instead of writing your own.

| Need                                                                                  | Use                                                                                                                                                     |
| ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Call Rust                                                                             | the typed wrappers: `commands` in `src/core/api.ts` (shell) or `src/modules/<id>/api.ts`                                                                |
| Listen to Rust                                                                        | `on(events.x, handler)` from `src/core/api.ts`                                                                                                          |
| Show a Rust error                                                                     | `errorMessage(error)` from `src/core/api.ts`                                                                                                            |
| Read or change settings                                                               | `settings.value` and `settings.update(patch)` from `src/core/settings/settings.svelte.ts`                                                               |
| Text                                                                                  | `t('key')` from `src/core/i18n` (shell) or the module's `i18n.ts`                                                                                       |
| A module's name or description                                                        | `moduleText(id, 'name' \| 'description')` from `src/core/i18n`                                                                                          |
| A button, switch, modal, panel, chip or toggle chip, card, checkbox, file icon, field | `src/ui/`                                                                                                                                               |
| An empty list or a list with no match                                                 | `EmptyState` from `src/ui/`: a centred pictogram, a title, a line and a button                                                                          |
| A dialog over the overlay                                                             | `Modal` or `Dialog` from `src/ui/`: it moves itself to `.dialog-layer` of the overlay, and `base.css` blurs `.behind-dialogs` while the layer holds one |
| Content that is still loading                                                         | `Skeleton` from `src/ui/`: shapes of the coming content, with a shimmer                                                                                 |
| Drag a tile                                                                           | `dragWithGhost(event, element)` from `src/ui/dragGhost.ts`: the browser's drag image has black corners in the transparent overlay                       |
| Colours, radii, veil                                                                  | the tokens in `src/ui/tokens.css`                                                                                                                       |
| Match text against a search                                                           | `matchesAll` and `normalize` from `src/core/text.ts` (case and accents ignored)                                                                         |
| Show a time ("2 min ago"), a size ("1.2 GB") or a share                               | `formatWhen`, `formatBytes`, `formatPercent` and `formatCount` from `src/core/format.ts`                                                                |
| Copy a path                                                                           | `copyPath` from `src/core/clipboard.ts`                                                                                                                 |
| Feedback after an action                                                              | `toast.show(message)` from `src/core/toast.svelte.ts`                                                                                                   |
| Open a page, the hub or settings                                                      | `navigation.open(view)` from `src/app/navigation.svelte.ts`                                                                                             |
| Open the uninstall dialog                                                             | `dialogs.uninstall = true` from `src/app/dialogs.svelte.ts`                                                                                             |
| Module list, tab bar, launcher order                                                  | `modules`, `getModule`, `tabBarIds`, `launcherIds` from `src/core/modules`                                                                              |

## Modules

A module is a folder in `src/modules/<id>/`. Its manifest declares the **elements** it provides, and each element has its own switch in Settings → Modules:

| Element       | What it gives                                                                                           |
| ------------- | ------------------------------------------------------------------------------------------------------- |
| `page`        | a page in "All pages", and a tab if the user puts it in the bar                                         |
| `widget`      | a card on Home; the side column shrinks slightly to fit its cards, then scrolls                         |
| `search`      | results in the search bar                                                                               |
| `tool`        | a mini-app found by name in the search, opened over the overlay                                         |
| `contextMenu` | actions in the right-click menu of Explorer, run by the module's program (installable modules, Windows) |

```
manifest.ts          what the module is (id, icon, tint, order) and the elements it provides, below
<Name>Page.svelte    its page, loaded on first open (pages only)
<Name>Widget.svelte  its card on Home, if any (`widget: { slot: 'top' | 'main' | 'side', component }`)
<Name>Tool.svelte    its tool, if any (`tool`), receiving an optional `input`
<Action>.svelte      a control next to its first switch in Settings → Modules, if any (`settingsAction`)
<Name>Settings.svelte  a section of its own in Settings, if any (`settingsSection: { component, action? }`; `action.hidden()` hides its button, when the empty section already offers it)
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
- `order` places its group among the others: calculator 0, shortcuts 3, tools 5, apps 10, web 25, system commands 30, files 40, "Create a shortcut" 90.
- A module may give several providers as an array (shortcuts: its results, then "Create a shortcut" last).
- `exclusive: true` hides every other group when it answers (the calculator).
- `status` is a component shown above the results while the provider cannot answer fully yet. File search shows the progress of the first indexing there (`file-index://building`): until it ends, no file can be found.
- A result names its `primary` action (Enter) and may add a `secondary` one (Ctrl + Enter), a `path` to copy, a `confirm` question for actions that cannot be undone, or a `panel` component that replaces the preview buttons.

The shell asks every enabled provider at each keystroke and shows each answer as it arrives, so a slow provider never holds back the others; answers to older queries are dropped. A thin line under the search bar shows that a provider is still answering, once it takes more than 150 ms. Below 2 characters (except commands and calculations), it asks for more instead of searching.

### Tools

A tool is a mini-app with its own interface, such as a colour picker. The shell finds it by its name and `keywords`, in a "Tools" group before the applications, without any search code in the module. Enter opens it over the overlay, in a sheet the size of "What's new"; Escape or a click outside goes back to the search. `tools.open(id, input)` from `src/core/tools.svelte.ts` opens a tool from elsewhere, and the tool receives `input`. The calculator does both: its tool is found by name, and its search result for a calculation adds "Open in the calculator" with the calculation as `input`.

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

## Installable modules

Built-in modules live in `src/modules/` and ship with the app. Installable modules do not: the user installs them from Settings → Modules → Catalog (also reached from the "Add modules" tile of All pages), or from a `.arthex` file without the network, and can uninstall them. Each element of an installed module has its switch in Settings → Modules like any other.

A `.arthex` file can be installed three ways: from the catalog, with "Install from a file…", or by opening it in Explorer. The installer associates `.arthex` with Arthex (`bundle.fileAssociations`), so Windows starts it, or the running instance through the single-instance plugin, with the file's path. `opened.rs` checks the package at once; the overlay asks "Install this module?" (`modules_opened`) and installs it only on confirmation (`modules_answer_opened`).

- **Package** (`src-tauri/src/services/module_host/package.rs`): a `.arthex` file is a zip.
  - `files.json` holds the SHA-256 of every other file, and `files.json.sig` its minisign signature made with the update key (the `pubkey` of the updater). An unsigned, altered, missing, extra or unsafe file refuses the whole package before anything is written.
  - `module.json` gives the `id`, `version`, `api`, `platform` (`any` or `windows-x86_64`…), the path of its program (`native`), `background`, its `elements` and `tags`, and the `icon`, `tint` and `texts` (name and description per language) shown before installing an opened file.
  - `index.js` is its interface (its default export comes from `defineModule` of the SDK), `style.css` its styles.
- **Files:** unpacked in `<local data>/modules/<id>/`, replacing the previous version in one rename. What a module keeps lives in `<data>/module-data/<id>/`, which survives updates; uninstalling deletes it only when the user ticks "Also delete its data".
- **Serving** (`protocol.rs`): interface files only (`js`, `css`, `json`, `png`, `svg`, `woff2`), at `arthex-module://localhost/<id>/` (`http://arthex-module.localhost/<id>/` on Windows). The CSP allows scripts and styles from there and nowhere else besides the app. With `api: 8`, images and videos (`png`, `jpg`, `gif`, `webp`, `mp4`, `mov`, `webm`) a module's program put in `<data>/module-data/<id>/media/` are served at `<base>user-media/<file>`, with ranges so that a video is read as it plays.
- **Catalog** (`catalog.rs`): `catalog.json` and `catalog.json.sig` from the latest release of `arthreto/arthex-modules`. They are read when the Catalog tab opens or ↻ is pressed, and 40 s after launch only when modules are installed and updates install automatically. The last pair read is kept in `module-catalog/` and shown at once, signature checked, while the network answers. Each entry gives the module's texts, icon, tint, elements, tags (at most 8), right-click actions, the date of its last change, its preview image and one package per platform (or `any`). The image is downloaded only when its details are opened (`modules_preview`), and checked against the hash of the signed catalog.
- **Categories:** the Catalog tab filters by type of element and by category. The categories are the closed list `TAGS` of `src/core/modules/tags.ts`: a module picks among them in `tags` and can never add one.
  - Each one has an English id in code (`math`, `games`…) and its name in every language under `settings.modules.tags.<id>` of `src/core/locales/`.
  - `defineModule` of the SDK refuses an unknown tag, so `arthex pack` fails. The interface ignores unknown tags of the catalog (`knownTags`).
  - Only the categories that hold a module are offered as filters; the search of the tab matches their names too.
  - To add a category: add its id to `TAGS` and its name to every locale file, in the same commit.
- **Interface** (`src/app/moduleHost.svelte.ts`): imports each installed `index.js`, checks its export with `adopt` (`src/core/modules/foreign.ts`) and adds the module to the registry. An installed module brings its own Svelte: each of its views is a `mount(target, props)` that `ForeignView` shows. It reaches the shell only through `window.__ARTHEX__` (`HostApi`): translations, toast, tools, navigation, and hiding or showing the overlay.
- **Program** (`program.rs`): the system side of a module is a program of its own, built for each platform, that the app relays without knowing what it does.
  - It talks in JSON lines on its standard input and output. It receives `{"type":"init","dataDir","locale"}`, then `{"type":"call","id","method","params"}`. It answers `{"type":"result","id","value"}` or `{"type":"error","id","message"}`, and may send `{"type":"event","name","payload"}`, which reaches the interface as `<id>://<name>`, and `{"type":"overlay","visible"}`, which shows or hides the overlay (`set_overlay_visible` of the crate).
  - The interface calls it with `window.__ARTHEX__.call(id, method, params)` (`modules_call`) and hears its events with `listen`.
  - `callHidden` (`modules_call_hidden`) hides the overlay, makes the call and brings the overlay back, without the usual 30 s limit. Rust runs all of it, since the page is suspended while the overlay is hidden. The screen then shows what is under the overlay, for a program that reads it; the program can also take the focus, and open windows of its own, as the Color module's picker does.
  - It starts on its first call and stops after a minute without one. With `background`, it runs as long as its module is active and comes back up to 3 times if it stops by itself. It never runs while its module is off, and it is stopped before an update or an uninstall.
  - It starts without arguments and without a window, in its data folder, and must exit when its standard input closes. A program scaled by Windows sees wrong coordinates: one that reads the screen or opens windows declares itself DPI aware first.
- **Versions:** `HOST_API` in `package.rs` versions `HostApi`. Only add to it, or raise the version. Each module declares the oldest version it works with (`api` in its manifest, 1 by default), so a new version never forces an update on modules that do not use it. One that asks for more than the app offers is refused with "Update Arthex first".
- **What goes where:** a feature of a module belongs in the module, with its libraries: the interface's are bundled in its package, the program's are in its `native/` Cargo project. Arthex only offers generic bricks, never a feature of one module, so that using a module costs nothing to those who do not install it.
- **Development:** a debug build loads unsigned modules from `ARTHEX_DEV_MODULES=<folder>`, one built module per `<folder>/<id>/`. Release builds never do.

### Make an installable module

Installable modules live outside this repository, in [`arthreto/arthex-modules`](https://github.com/arthreto/arthex-modules), cloned with `--recurse-submodules`. That repository holds this one as a submodule (`arthex/`, moved with `git submodule update --remote arthex` in a commit of its own) and references the kit as `"@arthex/sdk": "file:arthex/sdk"`, next to `svelte`, `vite`, `@sveltejs/vite-plugin-svelte` and `@tauri-apps/cli`, so every module is built with a known version of the SDK. Its CI builds every module, and its Release workflow packs, signs and publishes them with the catalog.

1. A module is a folder like a built-in one. Its `manifest.ts` adds a `version` and its `locales`:

   ```ts
   import { defineModule } from '@arthex/sdk';

   export default defineModule({
     id: 'timer',
     version: '1.0.0',
     icon: 'M12 7v5l3 2M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z',
     tint: 'green',
     order: 50,
     tags: ['time', 'productivity'], // from TAGS in src/core/modules/tags.ts
     locales: import.meta.glob('./locales/*.json', { eager: true, import: 'default' }),
     tool: () => import('./TimerTool.svelte'),
   });
   ```

2. Its views use what `@arthex/sdk` exports: the UI kit of `src/ui/`, `moduleTranslator` for its texts (as in a built-in module), `toast`, `openTool`, `closeTool`, `navigate` and the formatting helpers. A search provider works as in a built-in module, without `panel`.
3. A system side is a Cargo project in `native/` that depends on `arthex-module` (`sdk/native/`). The interface calls it with `moduleNative('timer').call(method, params)` and hears it with `.on(event, handler)`:

   ```rust
   use arthex_module::Module;
   use serde_json::json;

   fn main() -> std::io::Result<()> {
       Module::new()
           .on("start", |context, params| Ok(json!({ "locale": context.locale(), "params": params })))
           .run()
   }
   ```

4. To add actions to the right-click menu of Explorer, set `api: 4`, give it a program and declare them. Each `label` is `contextMenu.<id>` in the locales. Without `icon`, the action shows the icon of Arthex; `icon` is a `.ico` of the module's folder.

   ```ts
   contextMenu: [
     { id: 'compress', targets: ['file', 'folder'] },
     { id: 'convert', targets: ['file'], extensions: ['.png', '.jpg'], icon: 'assets/convert.ico' },
   ],
   ```

   ```rust
   Module::new()
       .on_context_menu(|context, action, path| Ok(json!({ "action": action, "path": path.display().to_string() })))
       .run()
   ```

   - `targets` are `file`, `folder`, `folder-background` (the empty space of an open folder) and `drive`.
   - Arthex writes one verb per target in `HKCU\Software\Classes` (`context_menu.rs`) while the element is on, in the language of the interface. The click starts `arthex.exe --module-action <module> <action> <path>`, or reaches the running Arthex through the single-instance plugin, and the program receives the checked path. The overlay stays closed: to show it, the program calls `set_overlay_visible`.
   - Each selected item is one call.
   - `submenu: [{ id: 'jpeg', label: 'JPEG' }]` shows choices under the entry (`SubCommands`, up to 32); the program receives `<id>:<choice>`. It needs `api: 5`.
   - A program can come with files: `native/bundle/` is copied next to it, and `native/prepare.mjs`, when present, runs first to fetch them. A package holds 64 MB at most.
   - Windows 11 shows these actions under "Show more options". Its first menu only takes commands of a signed, packaged app, which Arthex is not.

5. `hints: () => [{ keys: ['←', '→'], label: t('hints.move') }]` shows the keys of its page in the bar at the bottom of the overlay, in place of those of the search results. `together: true` joins keys that are pressed together with `+`. An Arthex older than `api: 7` leaves them out, so a module can use them without asking for it.
6. The details shown in the Catalog tab come from the module itself:
   - version, languages, systems, size, categories, search keywords and right-click actions are read when it is packed, and the date from the last commit of its folder;
   - `catalog.examples` of its locales lists search examples separated by `|`, and `catalog.page` and `catalog.tool` say what its page and tool do;
   - an image named `preview.webp`, `preview.png` or `preview.jpg` next to `manifest.ts`, 2 MB at most, is published beside the package as `<id>-preview.<ext>`. It is shown at 400 × 250 and larger on a click, so make it 2560 × 1600. Without one, its icon is shown.
7. Develop it without signing: `npx arthex build <module> --out dev-modules`, then `ARTHEX_DEV_MODULES=<full path of dev-modules> npm run tauri dev`.
8. Release it, with `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` set as for app releases:
   - `npx arthex pack <module> --out dist` (add `--target <rust target>` for each platform when it has a program) writes `<id>-<version>-<platform>.arthex`, its preview and its catalog entry;
   - `npx arthex catalog dist` merges the entries into a signed `catalog.json`;
   - publish every `.arthex`, every preview, `catalog.json` and `catalog.json.sig` in the latest release of `arthreto/arthex-modules`.
9. To show a file the user picked, the program copies it into `<dataDir>/media/` and the interface loads `user-media/<file>` next to its own files (`api: 8`). It never reaches the rest of the disk.
10. A change to `HostApi` or to what `@arthex/sdk` exports raises `HOST_API` in Rust and `API` in `sdk/cli.js` together. A module that uses it sets `api` to that version in its `manifest.ts`.

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

| Event                   | Payload                 | Meaning                                                           |
| ----------------------- | ----------------------- | ----------------------------------------------------------------- |
| `overlay://shown`       | none                    | the overlay was just shown                                        |
| `overlay://hiding`      | none                    | clear the page, then call `overlay_hide`                          |
| `settings://changed`    | `Settings`              | settings changed, from any source                                 |
| `app://navigate`        | target id               | open `hub`, `settings` or a page                                  |
| `hotkey://held`         | key names               | modifiers held while recording (Windows)                          |
| `hotkey://recorded`     | key names               | recording finished, `[]` when cancelled (Windows)                 |
| `file-index://updated`  | none                    | a file index scan finished                                        |
| `file-index://building` | `{ items, percent }`    | progress of the first scan, then `null` when it ends              |
| `clipboard://changed`   | none                    | the clipboard history changed (Windows)                           |
| `updates://available`   | `{ version }`           | the check at launch found an update it does not install by itself |
| `updates://progress`    | `{ downloaded, total }` | download progress of an update                                    |
| `modules://changed`     | none                    | a module was installed, updated or uninstalled                    |
| `modules://opened`      | none                    | a `.arthex` file opened from Explorer was checked                 |

## Data files

Modules keep their data in JSON files of the app data folder, read and written with `storage::read_json` and `storage::write_json` (`src-tauri/src/app/storage.rs`). Writes are atomic, and a corrupt file is set aside instead of crashing.

| File                | Folder     | Content                                                                                                               |
| ------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------- |
| `settings.json`     | config     | settings (see below)                                                                                                  |
| `shortcuts.json`    | data       | shortcuts: site, app, folder or file, or command line; name, icon or pictogram, keyword, on Home and/or in the search |
| `recent.json`       | data       | recently used programs (path, name, time; never window titles)                                                        |
| `notes.json`        | data       | notes (id, title, Markdown text, time)                                                                                |
| `clipboard.bin`     | data       | the clipboard history, encrypted, only when the user asks to keep it                                                  |
| `whats-new.json`    | data       | notes of an update being installed, shown once after it                                                               |
| `file-search.json`  | config     | folders added to the file index and the exclusions                                                                    |
| `index.bin`         | local data | the file index cache, rebuilt from the drives when missing                                                            |
| `modules/<id>/`     | local data | an installed module, as its package held it                                                                           |
| `module-data/<id>/` | data       | what an installed module keeps                                                                                        |

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
  - The hook can miss a key-up while the focus moves to the overlay. Before each modifier press, the machine takes the modifiers Windows says are down (`GetAsyncKeyState`) and starts over if they differ, so the next press is never lost.
  - The hook callback does O(1) work. Windows silently removes slow hooks.
- **Taskbar icon** (`src-tauri/src/app/taskbar_search/`, setting `taskbarSearch`, off by default):
  - Windows has no API to add anything to its taskbar or search box. Its only search providers are web ones for the EEA, through an MSIX package and an HTTPS server, which queries must not reach.
  - The icon is a layered window of our own, with no webview, owned by `Shell_TrayWnd`, so it stays above the taskbar and goes with it.
  - `place.rs` decides where it goes: inside the Windows search box when it has room, otherwise (search hidden or shown as a button) in the free room after the apps. It hides rather than cover something, when the taskbar is full.
  - UI Automation reads the taskbar on a thread of its own, on a foreground change (and once more a second later, since the button of a new app has no event), a move of the taskbar or a change of settings or display. Nothing is polled. Explorer restarting destroys the window, which is then created again.
- The window sets `dragDropEnabled: false`. Otherwise Tauri takes every drag on Windows for a file drop, and the drags inside the page (Customize in All pages, Home shortcuts) show the "forbidden" cursor.
- The startup entry is managed by `tauri-plugin-autostart`. It launches the app with `--autostart`, so it starts silently. Debug builds never register it.
- Install is NSIS per user. `src-tauri/windows/hooks.nsh` removes the startup entry on uninstall.
- Data lives in the Tauri app folders named after the identifier `dev.arthreto.arthex`. The uninstaller offers to delete them.
- **Clipboard history** (`src-tauri/src/modules/clipboard/`):
  - A message-only window registered with `AddClipboardFormatListener` hears each copy, so nothing is polled.
  - Nothing is read while the module is off or paused. Copies marked `ExcludeClipboardContentFromMonitorProcessing`, `Clipboard Viewer Ignore` or `CanIncludeInClipboardHistory = 0` (password managers) are skipped.
  - Text only, 100 KB at most, 50 entries besides pinned ones, in memory. Turning the module off forgets everything.
  - The `keepClipboard` setting (off by default, in the module's dialog of Settings → Modules) also saves them to `clipboard.bin`, encrypted for the Windows account with DPAPI, and brings them back at launch. Turning it off deletes the file.
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
2. Write an annotated tag on `main`, then push it. Its message is a title line (`vX.Y.Z`), an empty line, then the changelog (format below): the workflow reads only what follows the title, and refuses a changelog that starts on the first line.

   ```bash
   git tag -a --cleanup=verbatim vX.Y.Z
   ```

   `--cleanup=verbatim` matters: by default Git drops the lines that start with `#`, so the headings would be lost and the workflow would refuse the tag.

   ```bash
   git push origin vX.Y.Z
   ```

   Or run the Release workflow by hand (Actions → Release → Run workflow, or through the API) with the changelog as its `notes` input: it releases the version of `package.json` and creates the tag itself.

3. Start it right after pushing, alongside CI: there is no need to wait for CI, since a build that fails publishes nothing. The release workflow checks the versions and the changelog, builds the installer, signs it in a separate step (the only one that sees the key; its actions are pinned to commits), then publishes the GitHub Release with `latest.json`.

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
