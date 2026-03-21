import type { ThemeDefinition } from '@paulbreuler/shell';

/**
 * Alder Grove dark theme.
 *
 * Maps Grove's --grove-* palette into the shell's --app-* token schema.
 * The shell theme store applies these as inline custom properties on :root,
 * overriding the stylesheet defaults so shell chrome (ActivityBar, StatusBar,
 * panels) inherits Grove's visual identity.
 *
 * Source of truth for hex values: src/app.css :root declarations.
 *
 * NOTE: Hex values are required here (not var(--grove-*) references) because
 * the shell theme store applies these as inline style properties on :root.
 * Inline styles need resolved values. See themes.test.ts for drift detection.
 */
export const GROVE_DARK: ThemeDefinition = {
  id: 'grove-dark',
  name: 'Grove Dark',
  description: 'Alder Grove dark theme with indigo accents.',
  base: 'dark',
  tokens: {
    // Surfaces (from --grove-surface-*)
    'bg-app': '#0f1117',
    'bg-surface': '#1a1d27',
    'bg-raised': '#242836',
    'bg-elevated': '#2a2e3c',
    'bg-overlay': '#2a2e3c',
    // Text (from --grove-text-*)
    'text-primary': '#e4e4e7',
    'text-secondary': '#a1a1aa',
    'text-muted': '#71717a',
    // Borders (from --grove-border-*)
    'border-default': '#2e3240',
    'border-subtle': '#232732',
    'border-emphasis': '#3e4350',
    // Brand / Accent (from --grove-accent)
    'brand-primary': '#6366f1',
    'brand-primary-hover': '#818cf8',
    'ring': '#6366f1',
  },
};
