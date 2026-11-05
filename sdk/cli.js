#!/usr/bin/env node
// Builds, signs and lists installable modules. Signing uses the update key of the app:
// TAURI_SIGNING_PRIVATE_KEY and TAURI_SIGNING_PRIVATE_KEY_PASSWORD, as for releases.
//
//   arthex build <module folder> --out <folder>     unsigned, for ARTHEX_DEV_MODULES=<folder>
//   arthex pack <module folder> [--out <folder>] [--target <rust target>]
//   arthex catalog <folder of packed modules>

import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { build } from 'vite';
import { moduleConfig } from './vite.js';
import { zip } from './zip.js';

/** `HOST_API` of the app this SDK comes with. */
const API = 5;
const ELEMENTS = ['page', 'widget', 'search', 'tool', 'contextMenu'];
const require = createRequire(import.meta.url);

function fail(message) {
  console.error(`arthex: ${message}`);
  process.exit(1);
}

function option(args, name) {
  const at = args.indexOf(name);
  return at === -1 ? undefined : args[at + 1];
}

/** `windows-x86_64`, as Rust's `std::env::consts` name the system it runs on. */
function platformOf(target) {
  if (target) {
    const system = ['windows', 'linux', 'darwin'].find((name) => target.includes(name));
    if (!system) fail(`unknown target ${target}`);
    return `${system === 'darwin' ? 'macos' : system}-${target.split('-')[0]}`;
  }
  const systems = { win32: 'windows', darwin: 'macos', linux: 'linux' };
  const archs = { x64: 'x86_64', arm64: 'aarch64' };
  return `${systems[process.platform] ?? process.platform}-${archs[process.arch] ?? process.arch}`;
}

function sign(file) {
  if (!process.env.TAURI_SIGNING_PRIVATE_KEY) fail('set TAURI_SIGNING_PRIVATE_KEY to sign');
  const tauri = require.resolve('@tauri-apps/cli/tauri.js');
  execFileSync(process.execPath, [tauri, 'signer', 'sign', file], { stdio: 'ignore' });
}

function walk(dir, prefix = '') {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const name = prefix ? `${prefix}/${entry.name}` : entry.name;
    return entry.isDirectory() ? walk(path.join(dir, entry.name), name) : [name];
  });
}

const sha256 = (content) => createHash('sha256').update(content).digest('hex');

/** The program in `native/`, built for the target, copied as `native/<id>[.exe]`. */
function buildProgram(dir, staging, id, target) {
  const cargo = path.join(dir, 'native', 'Cargo.toml');
  if (!fs.existsSync(cargo)) return null;
  const targetArgs = target ? ['--target', target] : [];
  execFileSync('cargo', ['build', '--release', '--manifest-path', cargo, ...targetArgs], {
    stdio: 'inherit',
  });
  const name = /^\s*name\s*=\s*"([^"]+)"/m.exec(fs.readFileSync(cargo, 'utf8'))?.[1];
  if (!name) fail('native/Cargo.toml has no package name');
  const extension = platformOf(target).startsWith('windows') ? '.exe' : '';
  const prepare = path.join(dir, 'native', 'prepare.mjs');
  if (fs.existsSync(prepare)) execFileSync(process.execPath, [prepare], { stdio: 'inherit' });
  const built = path.join(dir, 'native', 'target', target ?? '', 'release', `${name}${extension}`);
  const packed = `native/${id}${extension}`;
  fs.mkdirSync(path.join(staging, 'native'), { recursive: true });
  fs.copyFileSync(built, path.join(staging, packed));
  const bundle = path.join(dir, 'native', 'bundle');
  if (fs.existsSync(bundle)) fs.cpSync(bundle, path.join(staging, 'native'), { recursive: true });
  return packed;
}

/** Its right-click actions, with their labels in each language and their icons copied. */
function contextMenuOf(dir, staging, module, native) {
  const actions = module.contextMenu;
  if (!actions) return undefined;
  if (module.api < 4) fail('a right-click action needs api: 4 in manifest.ts');
  if (!native) fail('a right-click action needs a program in native/, which runs it');
  if (module.api < 5 && actions.some((action) => action.submenu)) {
    fail('a submenu needs api: 5 in manifest.ts');
  }
  return actions.map((action) => {
    const labels = Object.fromEntries(
      Object.entries(module.locales).flatMap(([language, messages]) => {
        const label = messages.contextMenu?.[action.id];
        return label ? [[language, label]] : [];
      }),
    );
    if (Object.keys(labels).length === 0) fail(`no "contextMenu.${action.id}" in the locales`);
    if (action.icon) {
      if (!action.icon.toLowerCase().endsWith('.ico')) fail(`${action.icon}: Windows needs a .ico`);
      const source = path.join(dir, action.icon);
      if (!fs.existsSync(source)) fail(`${action.icon} is missing`);
      fs.mkdirSync(path.dirname(path.join(staging, action.icon)), { recursive: true });
      fs.copyFileSync(source, path.join(staging, action.icon));
    }
    return { ...action, labels };
  });
}

/** Builds the interface and the program of a module, and writes its `module.json`. */
async function stage(dir, target) {
  const staging = fs.mkdtempSync(path.join(os.tmpdir(), 'arthex-module-'));
  await build(moduleConfig(dir, staging));
  const module = (await import(pathToFileURL(path.join(staging, 'index.js')).href)).default;
  for (const field of ['id', 'version', 'icon', 'tint']) {
    if (typeof module?.[field] !== 'string') fail(`manifest.ts must use defineModule (no ${field})`);
  }
  if (module.api > API) fail(`this module needs API ${module.api}, and this SDK offers ${API}`);
  const native = buildProgram(dir, staging, module.id, target);
  const platform = native ? platformOf(target) : 'any';
  const contextMenu = contextMenuOf(dir, staging, module, native);
  const info = {
    id: module.id,
    version: module.version,
    api: module.api,
    platform,
    ...(native && { native }),
    background: module.background,
    elements: ELEMENTS.filter((element) => module[element] !== undefined),
    ...(contextMenu && { contextMenu }),
    tags: module.tags ?? [],
    icon: module.icon,
    tint: module.tint,
    texts: Object.fromEntries(
      Object.entries(module.locales).map(([language, messages]) => [
        language,
        { name: messages.name ?? module.id, description: messages.description ?? '' },
      ]),
    ),
  };
  fs.writeFileSync(path.join(staging, 'module.json'), JSON.stringify(info, null, 2));
  return { staging, module, info };
}

async function buildUnsigned(args) {
  const dir = path.resolve(args[0] ?? fail('which module folder?'));
  const out = path.resolve(option(args, '--out') ?? fail('--out <folder> is needed'));
  const { staging, module } = await stage(dir, undefined);
  const target = path.join(out, module.id);
  fs.rmSync(target, { recursive: true, force: true });
  fs.mkdirSync(out, { recursive: true });
  fs.cpSync(staging, target, { recursive: true });
  fs.rmSync(staging, { recursive: true, force: true });
  console.log(target);
}

async function pack(args) {
  const dir = path.resolve(args[0] ?? fail('which module folder?'));
  const out = path.resolve(option(args, '--out') ?? path.join(dir, 'dist'));
  const target = option(args, '--target');
  const { staging, module, info } = await stage(dir, target);
  const platform = info.platform;

  const files = walk(staging).sort();
  const hashes = Object.fromEntries(
    files.map((file) => [file, sha256(fs.readFileSync(path.join(staging, file)))]),
  );
  fs.writeFileSync(path.join(staging, 'files.json'), JSON.stringify(hashes, null, 2));
  sign(path.join(staging, 'files.json'));

  const name = `${module.id}-${module.version}-${platform}`;
  const archive = zip(
    [...files, 'files.json', 'files.json.sig'].map((file) => [
      file,
      fs.readFileSync(path.join(staging, file)),
    ]),
  );
  fs.mkdirSync(out, { recursive: true });
  fs.writeFileSync(path.join(out, `${name}.arthex`), archive);
  const entry = { ...info, file: `${name}.arthex`, size: archive.length };
  fs.writeFileSync(path.join(out, `${name}.json`), JSON.stringify(entry, null, 2));
  fs.rmSync(staging, { recursive: true, force: true });
  console.log(`${name}.arthex`);
}

const newer = (a, b) => {
  const [x, y] = [a, b].map((version) => version.split(/[.-]/).map((part) => Number.parseInt(part, 10) || 0));
  for (let at = 0; at < 3; at += 1) if (x[at] !== y[at]) return x[at] > y[at];
  return false;
};

/** Merges the `<name>.json` that `pack` wrote next to each package into a signed `catalog.json`. */
function catalog(args) {
  const dir = path.resolve(args[0] ?? fail('which folder of packed modules?'));
  const modules = new Map();
  for (const file of fs
    .readdirSync(dir)
    .filter((name) => name.endsWith('.json') && !name.startsWith('catalog'))) {
    const entry = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
    const known = modules.get(entry.id);
    if (known && newer(known.version, entry.version)) continue;
    const packages = known?.version === entry.version ? known.packages : {};
    packages[entry.platform] = { file: entry.file, size: entry.size };
    const { id, version, api, icon, tint, elements, tags, texts } = entry;
    modules.set(id, { id, version, api, icon, tint, elements, tags, texts, packages });
  }
  const file = path.join(dir, 'catalog.json');
  fs.writeFileSync(file, JSON.stringify({ modules: [...modules.values()] }, null, 2));
  sign(file);
  console.log(`catalog.json: ${modules.size} module(s)`);
}

const [command, ...args] = process.argv.slice(2);
if (command === 'build') await buildUnsigned(args);
else if (command === 'pack') await pack(args);
else if (command === 'catalog') catalog(args);
else fail('usage: arthex build|pack <module folder> | arthex catalog <folder>');
