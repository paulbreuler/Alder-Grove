import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, it, expect } from 'vitest';
import { GROVE_DARK } from './themes';

describe('GROVE_DARK', () => {
  it('has required ThemeDefinition fields', () => {
    expect(GROVE_DARK.id).toBe('grove-dark');
    expect(GROVE_DARK.name).toBe('Grove Dark');
    expect(GROVE_DARK.base).toBe('dark');
    expect(GROVE_DARK.tokens).toBeDefined();
  });

  it('provides surface tokens', () => {
    expect(GROVE_DARK.tokens['bg-app']).toBe('#0f1117');
    expect(GROVE_DARK.tokens['bg-surface']).toBe('#1a1d27');
    expect(GROVE_DARK.tokens['bg-raised']).toBe('#242836');
    expect(GROVE_DARK.tokens['bg-elevated']).toBe('#2a2e3c');
  });

  it('provides text tokens', () => {
    expect(GROVE_DARK.tokens['text-primary']).toBe('#e4e4e7');
    expect(GROVE_DARK.tokens['text-secondary']).toBe('#a1a1aa');
    expect(GROVE_DARK.tokens['text-muted']).toBe('#71717a');
  });

  it('provides border tokens', () => {
    expect(GROVE_DARK.tokens['border-default']).toBe('#2e3240');
    expect(GROVE_DARK.tokens['border-subtle']).toBe('#232732');
    expect(GROVE_DARK.tokens['border-emphasis']).toBe('#3e4350');
  });

  it('provides brand/accent tokens', () => {
    expect(GROVE_DARK.tokens['brand-primary']).toBe('#6366f1');
    expect(GROVE_DARK.tokens['brand-primary-hover']).toBe('#818cf8');
    expect(GROVE_DARK.tokens['ring']).toBe('#6366f1');
  });

  it('all token values are non-empty strings', () => {
    for (const [key, value] of Object.entries(GROVE_DARK.tokens)) {
      expect(value, `token "${key}" should be non-empty`).toBeTruthy();
      expect(typeof value, `token "${key}" should be string`).toBe('string');
    }
  });

  // Token mapping: --app-* suffix -> --grove-* CSS variable name
  const TOKEN_TO_CSS_VAR: Record<string, string> = {
    'bg-app': '--grove-surface-base',
    'bg-surface': '--grove-surface-elevated',
    'bg-raised': '--grove-surface-sunken',
    'bg-elevated': '--grove-surface-overlay',
    'bg-overlay': '--grove-surface-overlay',
    'text-primary': '--grove-text-primary',
    'text-secondary': '--grove-text-secondary',
    'text-muted': '--grove-text-muted',
    'border-default': '--grove-border-default',
    'border-subtle': '--grove-border-subtle',
    'border-emphasis': '--grove-border-strong',
    'brand-primary': '--grove-accent',
    'brand-primary-hover': '--grove-accent-hover',
    'ring': '--grove-accent',
  };

  it('token values match --grove-* declarations in app.css', () => {
    const css = readFileSync(resolve(__dirname, '../app.css'), 'utf-8');

    for (const [appKey, groveVar] of Object.entries(TOKEN_TO_CSS_VAR)) {
      const regex = new RegExp(`${groveVar}:\\s*([^;]+);`);
      const match = css.match(regex);
      expect(match, `${groveVar} not found in app.css`).toBeTruthy();

      const cssValue = match![1].trim();
      const themeValue = GROVE_DARK.tokens[appKey];
      expect(themeValue, `token "${appKey}" should match ${groveVar}`).toBe(
        cssValue,
      );
    }
  });
});
