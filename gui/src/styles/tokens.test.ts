// The theme's values are the stid design system's (D-046) and its faces ship inside the app (D-047).
// The stylesheets are read as text: jsdom computes no custom properties, and what is asserted here
// is exactly what the files declare.
import { expect, test } from 'vitest';
import main from '../main.ts?raw';
import fonts from './fonts.css?raw';
import tokens from './tokens.css?raw';

/** The custom properties declared in the first `:root { … }` at or after `opener`. */
function declarations(css: string, opener: RegExp): Record<string, string> {
  const start = css.search(opener);
  expect(start, `no block matching ${opener}`).toBeGreaterThanOrEqual(0);
  const open = css.indexOf('{', css.indexOf(':root', start));
  const body = css.slice(open + 1, css.indexOf('}', open));
  return Object.fromEntries([...body.matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)].map((m) => [m[1], m[2].trim()]));
}

const light = declarations(tokens, /^:root\s*\{/m);
const dark = declarations(tokens, /@media \(prefers-color-scheme: dark\)/);

test('AC-119 (RENDER): the window takes Catppuccin Latte in light and Mocha in dark, with the peach accent', () => {
  expect(light).toMatchObject({
    '--bg': '#eff1f5',
    '--bg-subtle': '#e6e9ef',
    '--text': '#4c4f69',
    '--accent': '#fe640b',
    '--accent-text': '#11111b',
  });
  expect(dark).toMatchObject({
    '--bg': '#1e1e2e',
    '--bg-subtle': '#181825',
    '--text': '#cdd6f4',
    '--accent': '#fab387',
    '--accent-text': '#11111b',
  });
});

test('AC-119 (RENDER): the type badges keep the colours D-045 gave them', () => {
  expect(light).toMatchObject({
    '--kind-prg-text': '#3b6a2a',
    '--kind-8k-text': '#7a5600',
    '--kind-16k-text': '#8a4535',
    '--kind-ult-text': '#6f3d86',
    '--kind-xb-text': '#4d42a0',
  });
  expect(dark).toMatchObject({
    '--kind-prg-text': '#9ad284',
    '--kind-8k-text': '#f2c84b',
    '--kind-16k-text': '#d8988a',
    '--kind-ult-text': '#c49ad8',
    '--kind-xb-text': '#aaa1ec',
  });
});

test('AC-121 (RENDER): the five raster bands are Catppuccin red, peach, yellow, green and blue in each theme', () => {
  const bands = ['red', 'peach', 'yellow', 'green', 'blue'].map((b) => `--band-${b}`);
  expect(bands.map((b) => light[b])).toEqual(['#d20f39', '#fe640b', '#df8e1d', '#40a02b', '#1e66f5']);
  expect(bands.map((b) => dark[b])).toEqual(['#f38ba8', '#fab387', '#f9e2af', '#a6e3a1', '#89b4fa']);
});

test('AC-120 (RENDER): text is set in Geist and figures in Geist Mono', () => {
  expect(light['--font-ui']).toMatch(/^['"]Geist Variable['"],/);
  expect(light['--font-mono']).toMatch(/^['"]Geist Mono Variable['"],/);
});

test('AC-120 (RENDER): both faces ship inside the app and nothing loads from the network', () => {
  const files = Object.keys(import.meta.glob('../fonts/*.woff2', { query: '?url', eager: true }));
  const named = [...fonts.matchAll(/url\(['"]?([^'")]+)['"]?\)/g)].map((m) => m[1]);
  const families = [...fonts.matchAll(/font-family:\s*['"]([^'"]+)['"]/g)].map((m) => m[1]);

  expect(named).toHaveLength(4);
  expect(named.filter((u) => !files.includes(u))).toEqual([]);
  expect(new Set(families)).toEqual(new Set(['Geist Variable', 'Geist Mono Variable']));
  expect(main).toMatch(/import ['"]\.\/styles\/fonts\.css['"];/);
  for (const css of [tokens, fonts]) {
    expect(css).not.toMatch(/https?:/);
    expect(css).not.toMatch(/@import/);
  }
});
