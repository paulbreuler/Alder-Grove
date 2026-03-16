/**
 * Generate TypeScript types from grove-domain Rust types via ts-rs.
 *
 * 1. Runs cargo test to trigger ts-rs export to crates/grove-domain/bindings/
 * 2. Copies generated .ts files to src/generated/
 * 3. Creates barrel index.ts re-exporting all types
 *
 * Cross-platform (no POSIX shell dependencies).
 */
import { execSync } from 'node:child_process';
import { cpSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const BINDINGS_DIR = 'crates/grove-domain/bindings';
const OUTPUT_DIR = 'src/generated';

// Step 1: Run ts-rs export tests
try {
  execSync('cargo test -p grove-domain --features ts -- export_bindings', {
    stdio: ['ignore', 'ignore', 'ignore'],
  });
} catch {
  console.error('Failed to run cargo test for ts-rs export');
  process.exit(1);
}

// Step 2: Copy bindings to src/generated/
mkdirSync(OUTPUT_DIR, { recursive: true });
cpSync(BINDINGS_DIR, OUTPUT_DIR, { recursive: true });

// Step 3: Generate barrel file
const files = readdirSync(OUTPUT_DIR)
  .filter(f => f.endsWith('.ts') && f !== 'index.ts')
  .sort();

const lines = [
  '// Auto-generated barrel file — do not edit manually.',
  '// Regenerate with: pnpm generate:types',
  '',
  ...files.map(f => `export * from './${f.replace('.ts', '')}';`),
  '',
];

writeFileSync(join(OUTPUT_DIR, 'index.ts'), lines.join('\n'));
console.log(`✓ ${files.length} TypeScript types generated in ${OUTPUT_DIR}/`);
