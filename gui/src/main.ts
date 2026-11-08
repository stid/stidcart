import { mount } from 'svelte';
import App from './App.svelte';
import { host } from './lib/host';
import './styles/fonts.css';
import './styles/tokens.css';

export default mount(App, { target: document.getElementById('app')!, props: { host } });
