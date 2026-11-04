<p align="center">
  <img src="src/assets/logo.png" alt="Arthex" width="96" height="96">
</p>

<h1 align="center">Arthex</h1>

<p align="center">A fast overlay for your shortcuts, files and everyday tools. Press <kbd>Shift</kbd> + <kbd>Win</kbd>.</p>

Arthex lives in the system tray and opens on top of everything. It shows as a full-screen translucent layer that blends into the desktop.

- Search files and folders instantly.
- Launch apps and websites.
- Calculate and run system commands.
- See what fills your disk.
- Keep notes and your clipboard history at hand.

It is built for Windows 11. macOS and Linux get a simplified version.

> **Status:** early development. The overlay, tray, settings, themes and languages work. The pages below arrive step by step.

## Features

|               |                                                                                                                         |
| ------------- | ----------------------------------------------------------------------------------------------------------------------- |
| **Search**    | Files and folders (own index, much faster than Windows Search), apps, websites, calculator (`=`), system commands (`>`) |
| **Home**      | Shortcuts to websites and software, recent apps, storage, notes and last copied item                                    |
| **FileSize**  | Folder tree, treemap and largest files, with deletion to the Recycle Bin                                                |
| **Notes**     | Markdown notes (headings, bold, italic, checkboxes) with Preview and Edit modes                                         |
| **Clipboard** | Text history kept in memory only, never written to disk                                                                 |
| **System**    | Lock, sleep, restart, shut down, scheduled shutdown                                                                     |
| **All pages** | Every page in one launcher. Pin your favourites to the top, choose the tab bar, turn pages off.                         |
| **Updates**   | Signed automatic updates from GitHub Releases, with release notes at the next launch                                    |

Languages: English and French.

## Install

1. Download `Arthex_x.y.z_x64-setup.exe` from the [Releases](https://github.com/arthreto/Arthex/releases) page.
2. Run it. No administrator rights are needed, and you can choose the install folder.

The installer is not code-signed yet, so Windows SmartScreen may warn you: click **More info → Run anyway**.

To uninstall, use the tray menu (**Uninstall…**) or Windows Settings → Apps.

## Use

| Keys                                         | Action                                        |
| -------------------------------------------- | --------------------------------------------- |
| <kbd>Shift</kbd> + <kbd>Win</kbd>            | Open or close Arthex (changeable in Settings) |
| <kbd>↑</kbd> <kbd>↓</kbd> · <kbd>Enter</kbd> | Choose and open a result                      |
| <kbd>Ctrl</kbd> + <kbd>Enter</kbd>           | Open the containing folder                    |
| <kbd>Ctrl</kbd> + <kbd>Tab</kbd>             | Next tab                                      |
| <kbd>Esc</kbd>                               | Clear the search, then close                  |

Arthex never needs administrator rights and sends no telemetry. The network is used only for website icons and update checks.

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

`npm run tauri build` writes the installer to `src-tauri/target/release/bundle/`.

Checks run by CI:

```bash
npm run check && npm run lint && npm test && npm run i18n:check
```

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

## Architecture

- **Stack:** [Tauri 2](https://tauri.app) (Rust) with a [Svelte 5](https://svelte.dev) + TypeScript interface.
- **`src/app/`:** the shell (overlay, search bar, tab bar, All pages, settings).
- **`src/core/`:** typed Rust API, settings, internationalisation and the module registry.
- **`src/ui/`:** shared components and theme tokens (dark and light).
- **`src/modules/<id>/`:** one folder per page or feature, discovered automatically.
- **`src-tauri/src/app/`:**
  - settings store (owned by Rust);
  - overlay window;
  - tray, startup entry, logging;
  - the commands the interface may call.
- **Security:** the window may only call the commands listed in `src-tauri/capabilities/main.json`, under a strict Content Security Policy.

### Create a module

1. Create `src/modules/<id>/manifest.ts`:
   ```ts
   export default defineModule({
     id: 'timer',
     kind: 'page',
     icon: '<svg path>',
     tint: 'green',
     order: 50,
     page: () => import('./TimerPage.svelte'),
   });
   ```
2. Write `TimerPage.svelte` with components from `src/ui/`.
3. Add `locales/en.json` with at least `name` and `description`, plus one file per other language.
4. Add `i18n.ts`: `export const t = moduleTranslator<typeof en>('timer');`.
5. Run `npm run i18n:check`. The module appears in All pages.

### Add a language

```bash
npm run i18n:new de
```

This creates every `de.json` from English. Translate them, then check that nothing is missing:

```bash
npm run i18n:check
```

## License

[PolyForm Noncommercial 1.0.0](LICENSE).

- You may use, study and modify Arthex, as long as you credit the author.
- Commercial use is not allowed, including for modified versions.
