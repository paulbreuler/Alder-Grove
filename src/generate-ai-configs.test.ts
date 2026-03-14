import { describe, expect, it } from 'vitest';

import {
  extractSection,
  parseSkillFrontmatter,
  prependGeneratedHeader,
  stripFrontmatter,
  renderCodexSkillWrapper,
} from '../scripts/generate-ai-configs';

describe('extractSection', () => {
  it('returns a markdown section body by heading', () => {
    const content = `# Root

## Overview

Hello world.

## Commands

\`pnpm test\`
`;

    expect(extractSection(content, 'Overview')).toBe(`## Overview

Hello world.`);
  });

  it('returns null when the heading is missing', () => {
    expect(extractSection('# Root', 'Missing')).toBeNull();
  });
});

describe('parseSkillFrontmatter', () => {
  it('extracts name and description from YAML frontmatter', () => {
    const content = `---
name: code-review
description: Review branch changes
---

# /code-review
`;

    expect(parseSkillFrontmatter(content)).toEqual({
      name: 'code-review',
      description: 'Review branch changes',
    });
  });
});

describe('stripFrontmatter', () => {
  it('removes leading YAML frontmatter and preserves the body', () => {
    const content = `---
paths:
  - "src/**/*.tsx"
---

# Frontend Rules

Use React.
`;

    expect(stripFrontmatter(content)).toBe(`# Frontend Rules

Use React.
`);
  });
});

describe('prependGeneratedHeader', () => {
  it('keeps YAML frontmatter at the top of the file', () => {
    const content = `---
name: code-review
description: Review branch changes
---

# Body
`;

    const result = prependGeneratedHeader(content);

    expect(result.startsWith(`---
name: code-review`)).toBe(true);
    expect(result).toContain('<!-- GENERATED FROM .claude/ — DO NOT EDIT BY HAND -->');
  });
});

describe('renderCodexSkillWrapper', () => {
  it('renders a deterministic Codex compatibility wrapper', () => {
    expect(
      renderCodexSkillWrapper({
        skillDir: 'code-review',
        name: 'code-review',
        description: 'Review branch changes',
      }),
    ).toBe(`---
name: code-review
description: Review branch changes
---

# code-review (Codex compatibility wrapper)

Canonical instructions: \`.claude/skills/code-review/SKILL.md\`

When invoked:
1. Read \`.claude/skills/code-review/SKILL.md\`.
2. Follow that workflow and output format.
3. Resolve relative paths from \`.claude/skills/code-review/\`.
`);
  });
});
