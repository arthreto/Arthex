import { mount } from 'svelte';
import App from './app/App.svelte';
import { commands } from './core/api';
import './ui/tokens.css';
import './ui/base.css';

const target = document.getElementById('app');
if (!target) throw new Error('Missing #app element');

mount(App, { target });

/** For the debug log: the page cannot be see-through when the webview renders without the GPU. */
function noteRenderer() {
  const gl = document.createElement('canvas').getContext('webgl');
  const info = gl?.getExtension('WEBGL_debug_renderer_info');
  const renderer = info && gl ? String(gl.getParameter(info.UNMASKED_RENDERER_WEBGL)) : 'no webgl';
  void commands.appNoteRenderer(renderer).catch(() => undefined);
}
noteRenderer();
