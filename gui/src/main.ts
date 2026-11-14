import { mount } from 'svelte';
import App from './App.svelte';
import HexView from './lib/HexView.svelte';
import { hexHost, host, isHexWindow } from './lib/host';
import './styles/fonts.css';
import './styles/tokens.css';

const target = document.getElementById('app')!;

// One page serves both windows; the hex viewer's is told apart by its label.
export default isHexWindow
  ? mount(HexView, { target, props: { host: hexHost } })
  : mount(App, { target, props: { host } });
