import { mount } from 'svelte';
import App from './app/App.svelte';
import './ui/tokens.css';
import './ui/base.css';

const target = document.getElementById('app');
if (!target) throw new Error('Missing #app element');

mount(App, { target });
