import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const here = path.dirname(fileURLToPath(import.meta.url));
const shellI18n = path.join(here, '../src/core/i18n/index.ts');
const moduleI18n = path.join(here, 'i18n.svelte.ts');

/**
 * Builds the module in `dir` (its `manifest.ts`) into `outDir`: `index.js`, its chunks and
 * `style.css`. The UI kit of the SDK reads its texts through the module's i18n instead of the
 * shell's, and the module carries its own Svelte.
 */
export function moduleConfig(dir, outDir) {
  return {
    configFile: false,
    root: dir,
    logLevel: 'warn',
    plugins: [
      svelte({ configFile: false, preprocess: vitePreprocess() }),
      {
        name: 'arthex-module-i18n',
        enforce: 'pre',
        async resolveId(source, importer, options) {
          const resolved = await this.resolve(source, importer, { ...options, skipSelf: true });
          return resolved?.id === shellI18n ? moduleI18n : null;
        },
      },
    ],
    resolve: {
      alias: { '@arthex/sdk': path.join(here, 'index.ts') },
      dedupe: ['svelte'],
    },
    build: {
      outDir,
      emptyOutDir: true,
      target: 'es2022',
      cssCodeSplit: false,
      rolldownOptions: { output: { chunkFileNames: '[name]-[hash].js' } },
      lib: {
        entry: path.join(dir, 'manifest.ts'),
        formats: ['es'],
        fileName: () => 'index.js',
        cssFileName: 'style',
      },
    },
  };
}
