# CLAUDE.md

Rules and context for anyone (human or AI) working on Arthex. Read it fully before changing anything.

Arthex is a fast, secure overlay utility. **Shift + Win** opens a full-screen translucent overlay with search, shortcuts, everyday tools and pages such as FileSize, Notes and Clipboard. It lives in the tray and starts with the system. Windows 11 comes first; macOS and Linux build a simplified version. Mobile is out of scope.

## Non-negotiable workflow

1. **Mockup first.** Any visible change (new screen, layout, component, wording) is made in the mockup first and **validated by the owner before the software is touched**. Then the app implements exactly what was validated, so the mockup always matches the app.
   - Mockup: https://claude.ai/artifact/8NjMTVR3fi2qD43JJUDika (Artifact of the "Design" type, one board per screen).
   - A purely internal change (refactor, fix, performance) needs no mockup.
2. **Commit and push often**, one logical change per commit.
   - Messages are in English, short and imperative ("Add clipboard history page").
   - Push to the working branch. Never open a pull request unless the owner asks for one.
3. **Every release has a changelog.** See "Releases" below. It is shown to users in the "What's new" view at their next launch, so it is never optional.
4. **English everywhere** in the repository: code, comments, commits, README, this file. The interface is translated through i18n. The owner writes in French; answer them in French.
5. **Only two markdown files exist:** `README.md` and `CLAUDE.md`. No `docs/`, no `CONTRIBUTING.md`, no notes files.

## Releases

- Tag `vX.Y.Z` on the default branch. The version must match in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
- The **Release description is the changelog** and is mandatory. It holds two sections, in this order, with the same items:

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

- The release workflow builds and signs the installers. It publishes `latest.json`, whose `notes` field is that description.
- At the first launch after an update (`settings.lastSeenVersion` differs from the running version), the app opens **What's new**: a large panel integrated with the overlay blur, showing the section of the active language. A release without a changelog is a broken release.

## Interface rules

These come from the owner's feedback on the mockup. Do not reintroduce what was rejected.

- **No superfluous text.** No explanatory subtitles, no "nothing is ever written to disk", no "no telemetry" taglines. A label says what a control does, nothing more.
- **No "AI-looking" gadgets:**
  - no shimmer or shine;
  - no glass or gradient buttons;
  - no gradient icon tiles, no glowing halos;
  - no decorative previews.
- Buttons are plain solid pills (`ui/Button.svelte`). The primary one is flat accent colour.
- Icons are sober: neutral translucent tile, thin border, a tinted stroked pictogram (`tint` of the module).
- **Overlay:** full screen on the monitor under the cursor, blur plus a dark veil fading towards the edges (`--veil`). It must feel integrated with the desktop, not like a window.
- **Modals** are simple:
  - the whole screen is dimmed and slightly blurred;
  - a plain 560 px panel holds a title, ✕, the fields, then Cancel / Confirm;
  - Escape or a click outside closes it.
- **Pinning** a page means it appears **first in "All pages"**. It does **not** add the page to the tab bar. `settings.favorites` and `settings.tabBar` are independent.
- **"All pages" hub:**
  - a launcher of sober 96 px icons;
  - a "Customize" mode with three drag zones: "In the bar" (max 6), "Other pages" and "Turned off";
  - Home is locked first in the bar.
  - **Settings is never a page.** It is the ⚙ icon next to ▦ "All pages".
- Features without a page (calculator, system commands, app launcher, file search…) are toggled in **Settings → Features**, not in the hub.
- Logo at the top left, next to the search bar. Layout from top to bottom: search bar, tab bar, page content, keyboard hints.
- Animations are short (opacity and scale) and respect "Reduce motion".
- **Nothing is hard-coded:**
  - visible strings go through i18n;
  - colours go through the tokens in `src/ui/tokens.css`, in both dark and light themes.

## Code rules

- Professional, small, single-purpose files. No dead code, no speculative abstractions, no unused files.
- **Few comments.** Only for a non-obvious _why_: a Win32 workaround, a security constraint, a platform quirk. Never paraphrase the code.
- Explicit English names.
- **Reuse before writing.**
  - Visual building blocks live in `src/ui/`; a module never redefines a button, switch, modal or card.
  - System access lives in Rust services.
  - Shared helpers live in `src/core/`.
- **TypeScript:**
  - `strict`, `noUncheckedIndexedAccess`, no `any`;
  - Svelte 5 runes only, with state in `*.svelte.ts` files.
- **Rust:**
  - edition 2024; `cargo fmt`; `clippy` pedantic with `-D warnings`;
  - no `unwrap` outside tests;
  - `unsafe` only in small, commented Win32 wrappers.
- Tests sit next to the code (`*.test.ts`, `#[cfg(test)]`). Test pure logic: layout rules, settings normalisation, the hotkey state machine, parsers, scoring.
- Dependencies:
  - Pin TypeScript to `~6.0.x`. TypeScript 7 breaks `svelte-check` and `typescript-eslint`.
  - Lock files are committed.

### Checks before every commit

```bash
npm run check && npm run lint && npm test && npm run i18n:check
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

CI runs the same checks and builds the Windows installer.

## Architecture

```
src/
  app/            shell: App, Overlay, SearchBar, TabBar, Hub, KeyHints, settings/
  core/           api.ts (typed commands and events), i18n/, settings/, modules/ (registry), appearance, toast
  ui/             shared components + tokens.css (themes) + base.css
  modules/<id>/   one folder per page or feature
src-tauri/
  build.rs        command list (ACL) + embeds the core locales for Rust
  capabilities/   what the window may call
  src/app/        setup, settings store, overlay window, tray, autostart, i18n, logging, commands
```

### Everything is a module

A page (Home, FileSize, Notes…) or a feature (calculator, system commands…) is a folder in `src/modules/<id>/`:

```
manifest.ts          defineModule({ id, kind: 'page' | 'feature', icon, tint, order, locked?, page? })
<Name>Page.svelte    loaded on first open (pages only)
locales/en.json      { "name": "…", "description": "…", … }, other languages alongside
i18n.ts              export const t = moduleTranslator<typeof en>('<id>')
```

- The registry discovers manifests with `import.meta.glob`. **Adding a module means adding a folder**; the shell is not edited.
- `id` is lowercase `[a-z0-9-]`, at most 32 characters, and never changes once released, because settings store it.
- Layout rules are pure functions in `src/core/modules/layout.ts`:
  - `tabBarIds`: home first, max 6, no disabled modules;
  - `launcherIds`: favourites, then enabled, then disabled;
  - `toggleFavorite`.
- A disabled module disappears everywhere: tab bar, search results, home widgets. Its Rust service stops.

### Settings

- Owned by Rust: `settings.json` in the app config folder, versioned and written atomically.
- The interface sends **partial patches** (`commands.settingsUpdate({ theme: 'dark' })`). Rust rejects unknown keys, normalises the result, then emits `settings://changed` to every listener.
- A new setting goes in three places, which must stay in sync:
  - `Settings` in `src-tauri/src/app/settings.rs`, with its default and normalisation;
  - `src/core/settings/types.ts`;
  - a test.
- A breaking change bumps `VERSION` and migrates old files.

### Adding a Rust command

1. Write it in `src-tauri/src/app/commands.rs` (or in the module's Rust file). Validate every input.
2. Add its name to `COMMANDS` in `src-tauri/build.rs`.
3. Add `allow-<name-with-dashes>` to `src-tauri/capabilities/main.json`.
4. Register it in `generate_handler!` in `src-tauri/src/lib.rs`.
5. Add a typed wrapper in `src/core/api.ts`.

Events shared by both sides are declared in `src-tauri/src/app/events.rs` and in `events` in `src/core/api.ts`.

### Internationalisation

- ICU messages (`intl-messageformat`).
  - Shell strings live in `src/core/locales/<lang>.json`.
  - Module strings live in `src/modules/<id>/locales/<lang>.json`, under the `modules.<id>.*` namespace.
- English is the reference and the fallback. Keys are typed from `en.json`, so a missing key does not compile.
- `npm run i18n:new <code>` creates every file for a new language. `npm run i18n:check` reports missing or unknown keys and mismatched variables.
- The tray menu is translated by Rust from the same `core/locales` files (`tray.*`), embedded at build time.
- Dates, sizes and numbers go through `Intl` in the active language.

## Security

- **Minimal capabilities:** only our commands and the plugin permissions actually used. No `shell` plugin.
- **Strict CSP:** no remote content in the webview.
- Every system action is a typed Rust command with validated input:
  - URLs are `http`/`https` only;
  - paths are canonicalised and must exist;
  - programs launch without arguments;
  - system commands are a closed enum.
- No admin rights, no telemetry. The network is used only for favicons and update checks.
- Logs never contain keystrokes, clipboard content or search queries.
- File deletion always goes to the Recycle Bin, after confirmation.
- Clipboard history stays in memory only and skips content excluded by password managers.

## Performance targets

| Metric          | Target                                                            |
| --------------- | ----------------------------------------------------------------- |
| Overlay opening | < 100 ms (the window is created hidden at startup and only shown) |
| Idle CPU        | ≈ 0 % (event-driven, no polling)                                  |
| Total RAM       | < 150 MB including the file index                                 |
| Search          | < 50 ms per keystroke                                             |
| Installer       | < 10 MB                                                           |

## Platform notes

- **Shift + Win** cannot be registered with `RegisterHotKey`, because it is made only of modifiers.
  - It uses a `WH_KEYBOARD_LL` hook on its own thread, driven by a pure, tested state machine.
  - It fires when Shift + Win is released with no other key in between, so Win + Shift + S still works.
  - A neutral `vkE8` key press is injected so that releasing Win does not open the Start menu.
  - The hook callback does O(1) work. Windows silently removes slow hooks.
- The startup entry is managed by `tauri-plugin-autostart`, launched with `--autostart` so the app starts silently. Debug builds never register it.
- Install is NSIS per user. `src-tauri/windows/hooks.nsh` removes the startup entry on uninstall.
- Data lives in the Tauri app folders named after the identifier `dev.arthreto.arthex`. The NSIS uninstaller offers to delete them.

## Roadmap

0. Mockup — done.
1. Skeleton: shell, UI kit, i18n, themes, settings, tray, autostart, overlay window, CI.
2. Onboarding, Shift + Win hook and recorder, "All pages" hub with Customize.
3. Home: website and software shortcuts, recent apps.
4. Search: file index, app launcher, calculator, system commands (including scheduled shutdown).
5. FileSize: tree, treemap, largest files, Recycle Bin.
6. Clipboard history and Markdown notes.
7. Signed auto-updates with What's new, then polish, accessibility and a performance pass.

Ideas for later, never started without the owner's go-ahead:

- web search keywords;
- unit and currency converter;
- colour picker;
- screenshot OCR;
- system monitor;
- work profiles;
- text snippets;
- audio output switcher;
- timer;
- MFT turbo indexing.

## License

PolyForm Noncommercial 1.0.0. Credit is required, modification is allowed and commercial use is never allowed, including for modified versions. Keep the `Required Notice` line in `LICENSE`.
