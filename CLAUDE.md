# CLAUDE.md

Rules for anyone (human or AI) working on Arthex. Read it fully before changing anything. How the code works and how to extend it (API, modules, commands, settings, languages, releases) is in [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md): read it before writing code.

Arthex is a fast, secure overlay utility. **Shift + Win** opens a full-screen translucent overlay with search, shortcuts, everyday tools and pages such as FileSize, Notes and Clipboard. It lives in the tray and starts with the system. Windows 11 comes first; macOS and Linux build a simplified version. Mobile is out of scope.

## Non-negotiable workflow

1. **Mockup first.** Any visible change (new screen, layout, component, wording) is made in the mockup first and **validated by the owner before the software is touched**. Then the app implements exactly what was validated, so the mockup always matches the app.
   - Mockup: https://claude.ai/artifact/8NjMTVR3fi2qD43JJUDika (Artifact of the "Design" type, one board per screen).
   - A purely internal change (refactor, fix, performance) needs no mockup.
2. **Commit and push often**, one logical change per commit.
   - Messages are in English, short and imperative ("Add clipboard history page").
   - Work and push directly on `main`. No feature or tool-named branches, no pull requests unless the owner asks.
   - Commits are authored as the owner (`arthreto <86776616+arthreto@users.noreply.github.com>`) and carry no co-author or session trailers.
3. **Every release has a changelog** as its description, in English and French, in the format given in `docs/DEVELOPMENT.md` → Releasing. Users see it in "What's new" at their next launch, so a release without one is a broken release.
   - Start the release right after pushing, alongside CI, never after waiting for it: its own Windows build fails on code that does not compile, and then nothing is published.
4. **English everywhere** in the repository: code, comments, commits, README, this file. The interface is translated through i18n. The owner writes in French; answer them in French.
5. **Three markdown files only**, each with one role. No other markdown file.
   - `README.md`: presentation, installation and credits, for users. Screenshots: one large Home image, then Search and FileSize only, in English, over a desktop wallpaper.
   - `docs/DEVELOPMENT.md`: how the code works and how to extend it, for developers. Update it in the same commit as any change it describes.
   - `CLAUDE.md`: the project rules.
6. **Run the checks** from `docs/DEVELOPMENT.md` → Checks before every commit.

## Interface rules

These come from the owner's feedback on the mockup. Do not reintroduce what was rejected.

- **No superfluous text.** No explanatory subtitles, no "nothing is ever written to disk", no "no telemetry" taglines. A label says what a control does, nothing more.
- **No "AI-looking" gadgets:**
  - no shimmer or shine, except on content that is still loading (`ui/Skeleton.svelte`);
  - no glass or gradient buttons;
  - no gradient icon tiles, no glowing halos;
  - no decorative previews.
- Buttons are plain solid pills (`ui/Button.svelte`). The primary one is flat accent colour.
- Icons are sober: neutral translucent tile, thin border, a tinted stroked pictogram (`tint` of the module).
- **Overlay:** full screen on the monitor under the cursor, blur plus a dark veil fading towards the edges (`--veil`). It must feel integrated with the desktop, not like a window.
- **Modals** are simple:
  - the whole screen is dimmed and strongly blurred, as behind "What's new";
  - a plain, slightly translucent 560 px panel holds a title, ✕, the fields, then Cancel / Confirm;
  - Escape or a click outside closes it.
  - "What's new" and tools are larger panels on the same layer (`ui/Dialog.svelte`): same dimming, blur and panel, never a look of their own.
- **Pinning** a page means it appears **first in "All pages"**. It does **not** add the page to the tab bar. `settings.favorites` and `settings.tabBar` are independent.
- **"All pages" hub:**
  - a launcher of sober 96 px icons, ending with an "Add modules" tile that opens Settings → Modules → Catalog;
  - a "Customize" mode with three drag zones: "In the bar" (max 6), "Other pages" and "Turned off";
  - Home is locked first in the bar.
  - **Settings is never a page.** It is the ⚙ icon next to ▦ "All pages".
- A module provides **elements**: a page, a Home card, search results, a tool. **Settings → Modules → Configuration** shows the icons of the modules grouped by type (Pages, Home cards, Search, Tools), grey when turned off. A click opens the module's dialog, with one switch per element. The hub's "Turned off" zone drives the same page switch.
- Logo at the top left, next to the search bar. Layout from top to bottom: search bar, tab bar, page content, keyboard hints.
- Animations are short (opacity and scale) and respect "Reduce motion".
- **Nothing that looks generated:** no imitation Windows icons or fake system UI in the mockup or the screenshots. Sample sites and apps always show their real icon (or Arthex's own icon), never coloured letters or made-up icons. No meta text such as "validated design" or "mockup" in anything users read.
- **Nothing is hard-coded:**
  - visible strings go through i18n;
  - colours go through the tokens in `src/ui/tokens.css`, in both dark and light themes.

## Code rules

- Professional, small, single-purpose files. No dead code, no speculative abstractions, no unused files.
- **Few comments.** Only for a non-obvious _why_: a Win32 workaround, a security constraint, a platform quirk. Never paraphrase the code.
- Explicit English names.
- **Modules carry their own weight.** A feature of a module goes in the module, and so do its libraries: bundled in its package, or in its `native/` program. Arthex gets generic bricks that any module can use, never a feature of one module. Before touching Arthex for a module, check that the module's program cannot do it.
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

## Security

- **Minimal capabilities:** only our commands and the plugin permissions actually used. No `shell` plugin.
- **Strict CSP:** no remote content in the webview. Installed modules are served from the app's own folder through the `arthex-module` protocol.
- **Installable modules are signed** with the update key. Rust checks the signature and every file of a package before writing anything. A module's program runs only while its module is active and starts without arguments or window (it may open its own), and the app only relays its messages.
- Every system action is a typed Rust command with validated input:
  - URLs are `http`/`https` only;
  - paths are canonicalised and must exist;
  - programs launch without arguments, except the command lines the user writes in their own shortcuts, run as typed through the system shell;
  - system commands are a closed enum.
- No admin rights, no telemetry. The network is used only for favicons, update checks, the module catalog and, in the Discord RPC module, the images of its preview (Discord's list of an application's images, and the addresses the user types).
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

CI measures the installer, the program and the memory used on Windows at every push (summary of the `windows` job). Check them after any change that could weigh on them.

## Roadmap

0. Mockup — done.
1. Skeleton: shell, UI kit, i18n, themes, settings, tray, autostart, overlay window, CI — done.
2. Onboarding, Shift + Win hook and recorder, "All pages" hub with Customize — done.
3. Home: website and software shortcuts (edit, delete with undo, reorder), recent apps, Settings → Features — done.
4. Search: file index, app launcher, calculator, system commands (including scheduled shutdown) — done.
5. FileSize: tree, treemap, largest files, Recycle Bin — done.
6. Clipboard history, Markdown notes and the Home cards — done.
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
