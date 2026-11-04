// Checks that every translation matches the English reference, or scaffolds a new language.
// Usage: node scripts/i18n.js check | node scripts/i18n.js new <code>
import { parse } from '@formatjs/icu-messageformat-parser';
import { existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const src = fileURLToPath(new URL('../src/', import.meta.url));
const shellDir = join(src, 'core/locales');
const moduleDirs = readdirSync(join(src, 'modules'))
  .map((id) => join(src, 'modules', id, 'locales'))
  .filter(existsSync);
const directories = [shellDir, ...moduleDirs];

const read = (file) => JSON.parse(readFileSync(file, 'utf8'));
const show = (file) => relative(src, file);

function flatten(tree, prefix = '', into = new Map()) {
  for (const [key, value] of Object.entries(tree)) {
    if (key.startsWith('$')) continue;
    if (typeof value === 'string') into.set(prefix + key, value);
    else flatten(value, `${prefix}${key}.`, into);
  }
  return into;
}

/** Names of the ICU arguments of a message, including those inside plural and select choices. */
function collect(elements, into) {
  for (const element of elements) {
    if (typeof element.value === 'string' && element.type !== 0 && element.type !== 7)
      into.add(element.value);
    for (const option of Object.values(element.options ?? {})) collect(option.value, into);
  }
  return into;
}

const variables = (message) => [...collect(parse(message), new Set())].sort().join(',');

function check() {
  const problems = [];
  for (const dir of directories) {
    const reference = flatten(read(join(dir, 'en.json')));
    for (const name of readdirSync(dir).filter((f) => f.endsWith('.json') && f !== 'en.json')) {
      const file = join(dir, name);
      const own = flatten(read(file));
      for (const [key, message] of reference) {
        if (!own.has(key)) problems.push(`${show(file)}: missing "${key}"`);
        else if (variables(own.get(key)) !== variables(message))
          problems.push(`${show(file)}: "${key}" must use the variables of the English text`);
      }
      for (const key of own.keys()) {
        if (!reference.has(key)) problems.push(`${show(file)}: unknown "${key}"`);
      }
    }
  }
  for (const name of readdirSync(shellDir)) {
    const meta = read(join(shellDir, name)).$meta;
    if (!meta?.code || !meta?.name || !meta?.nativeName)
      problems.push(`core/locales/${name}: incomplete "$meta"`);
  }
  if (problems.length > 0) {
    console.error(problems.join('\n'));
    process.exit(1);
  }
  console.log('Translations are complete.');
}

function create(code) {
  if (!/^[a-z]{2,3}(-[A-Z]{2})?$/.test(code ?? '')) {
    console.error('Usage: npm run i18n:new <code>   (for example: de, es, pt-BR)');
    process.exit(1);
  }
  for (const dir of directories) {
    const target = join(dir, `${code}.json`);
    if (existsSync(target)) continue;
    const messages = read(join(dir, 'en.json'));
    if (dir === shellDir) messages.$meta = { code, name: code, nativeName: code };
    writeFileSync(target, `${JSON.stringify(messages, null, 2)}\n`);
    console.log(`Created ${show(target)}`);
  }
  console.log('Translate these files and fill "$meta", then run: npm run i18n:check');
}

const [command, argument] = process.argv.slice(2);
if (command === 'check') check();
else if (command === 'new') create(argument);
else {
  console.error('Usage: node scripts/i18n.js check | new <code>');
  process.exit(1);
}
