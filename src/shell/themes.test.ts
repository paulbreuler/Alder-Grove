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
});
