#!/usr/bin/env tsx

import { execFileSync } from 'node:child_process';
import {
  type Dirent,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import prettier from 'prettier';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const GENERATED_HEADER = '<!-- GENERATED FROM .claude/ — DO NOT EDIT BY HAND -->\n\n';
const CHECK_MODE = process.argv.includes('--check');
const STYLEGUIDE_SECTIONS = [
  'Overview',
  'Architecture',
  'Development Practices',
  'Commands',
  'Documentation Structure',
  'What NOT to Build (v1)',
] as const;

type SkillFrontmatter = {
  name: string;
  description: string;
};

type GeneratedFile = {
  path: string;
  content: string;
};

function read(relativePath: string): string {
  return readFileSync(join(ROOT, relativePath), 'utf8');
}

function listDirectories(basePath: string, requiredChild: string): string[] {
  const absolutePath = join(ROOT, basePath);
  if (!existsSync(absolutePath)) {
    return [];
  }

  return readdirSync(absolutePath, { withFileTypes: true })
    .filter(
      (entry: Dirent) => entry.isDirectory() && existsSync(join(absolutePath, entry.name, requiredChild)),
    )
    .map((entry: Dirent) => entry.name)
    .sort();
}

function listMarkdownBasenames(basePath: string): string[] {
  const absolutePath = join(ROOT, basePath);
  if (!existsSync(absolutePath)) {
    return [];
  }

  return readdirSync(absolutePath)
    .filter((entry: string) => entry.endsWith('.md'))
    .map((entry: string) => entry.replace(/\.md$/, ''))
    .sort();
}

function extractFrontmatter(content: string): string | null {
  const match = content.match(/^---\n([\s\S]*?)\n---\n?/);
  return match ? match[1] : null;
}

export function stripFrontmatter(content: string): string {
  return content.replace(/^---\n[\s\S]*?\n---\n*/, '');
}

export function extractSection(content: string, heading: string): string | null {
  const escapedHeading = heading.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const expression = new RegExp(`(?:^|\\n)## ${escapedHeading}\\n([\\s\\S]*?)(?=\\n## (?!#)|$)`);
  const match = content.match(expression);

  if (!match) {
    return null;
  }

  const body = match[1].replace(/\n---\s*$/, '').trim();
  if (!body) {
    return null;
  }

  return `## ${heading}\n\n${body}`;
}

export function parseSkillFrontmatter(content: string): SkillFrontmatter {
  const frontmatter = extractFrontmatter(content);
  if (!frontmatter) {
    throw new Error('Skill file is missing YAML frontmatter.');
  }

  const name = frontmatter.match(/^name:\s*(.+)$/m)?.[1]?.trim() ?? '';
  const description = frontmatter.match(/^description:\s*(.+)$/m)?.[1]?.trim() ?? '';

  if (!name || !description) {
    throw new Error('Skill file must include name and description in frontmatter.');
  }

  return {
    name: name.replace(/^["']|["']$/g, ''),
    description: description.replace(/^["']|["']$/g, ''),
  };
}

function parseRulePaths(content: string): string[] {
  const frontmatter = extractFrontmatter(content);
  if (!frontmatter) {
    return [];
  }

  const lines = frontmatter.split('\n');
  const paths: string[] = [];
  let insidePaths = false;

  for (const line of lines) {
    if (/^paths:\s*$/.test(line)) {
      insidePaths = true;
      continue;
    }

    if (!insidePaths) {
      continue;
    }

    const pathMatch = line.match(/^\s*-\s*(.+)\s*$/);
    if (pathMatch) {
      paths.push(pathMatch[1].trim().replace(/^["']|["']$/g, ''));
      continue;
    }

    if (line.trim() !== '') {
      break;
    }
  }

  return paths;
}

export function prependGeneratedHeader(content: string): string {
  if (!content.startsWith('---\n')) {
    return GENERATED_HEADER + content;
  }

  const closingIndex = content.indexOf('\n---\n', 4);
  if (closingIndex === -1) {
    return GENERATED_HEADER + content;
  }

  const frontmatterEnd = closingIndex + '\n---\n'.length;
  const frontmatter = content.slice(0, frontmatterEnd);
  const rest = content.slice(frontmatterEnd).replace(/^\n/, '');
  return `${frontmatter}\n${GENERATED_HEADER}${rest}`;
}

export function renderCodexSkillWrapper({
  skillDir,
  name,
  description,
}: SkillFrontmatter & { skillDir: string }): string {
  return `---
name: ${name}
description: ${description}
---

# ${name} (Codex compatibility wrapper)

Canonical instructions: \`.claude/skills/${skillDir}/SKILL.md\`

When invoked:
1. Read \`.claude/skills/${skillDir}/SKILL.md\`.
2. Follow that workflow and output format.
3. Resolve relative paths from \`.claude/skills/${skillDir}/\`.
`;
}

async function formatMarkdown(content: string): Promise<string> {
  return prettier.format(content, {
    parser: 'markdown',
    proseWrap: 'preserve',
  });
}

async function renderGenerated(relativePath: string, content: string): Promise<GeneratedFile> {
  return {
    path: relativePath,
    content: await formatMarkdown(prependGeneratedHeader(content)),
  };
}

function ensureParent(relativePath: string): void {
  mkdirSync(dirname(join(ROOT, relativePath)), { recursive: true });
}

function writeGeneratedFiles(files: GeneratedFile[]): void {
  for (const file of files) {
    ensureParent(file.path);
    writeFileSync(join(ROOT, file.path), file.content, 'utf8');
  }
}

function removeStaleGeneratedDirectories(): void {
  const legacyAgentsPath = join(ROOT, '.agents/agents');
  if (existsSync(legacyAgentsPath)) {
    rmSync(legacyAgentsPath, { recursive: true, force: true });
  }
}

function stageGeneratedFiles(files: GeneratedFile[]): void {
  if (CHECK_MODE || !process.env.GIT_INDEX_FILE || files.length === 0) {
    return;
  }

  try {
    execFileSync('git', ['add', ...files.map((file) => file.path)], {
      cwd: ROOT,
      stdio: 'pipe',
    });
  } catch {
    // Best effort only.
  }
}

function generatedFilePaths(skillDirs: string[], ruleBasenames: string[]): string[] {
  return [
    'AGENTS.md',
    'GEMINI.md',
    '.gemini/styleguide.md',
    '.agents/skills/README.md',
    '.github/copilot-instructions.md',
    ...skillDirs.map((skillDir) => `.agents/skills/${skillDir}/SKILL.md`),
    ...ruleBasenames.map((ruleName) => `.github/instructions/${ruleName}.instructions.md`),
  ];
}

export function renderAgentsMarkdown({
  skillDirs,
  agentNames,
  commandNames,
}: {
  skillDirs: string[];
  agentNames: string[];
  commandNames: string[];
}): string {
  let content = '# Codex Repository Instructions\n\n';
  content += 'This repository keeps canonical assistant playbooks in `.claude/`.\n';
  content += 'Use the generated compatibility files in `.agents/skills/` to bridge those playbooks into Codex.\n\n';
  content += 'Project-wide guidance lives in `CLAUDE.md`.\n';
  content += 'Path-specific guidance lives in `.claude/rules/*.md`.\n\n';

  if (commandNames.length > 0) {
    content += '## Command Playbooks\n\n';
    content += 'When a user invokes a slash-style command, load the matching file from `.claude/commands/`.\n\n';
    for (const commandName of commandNames) {
      content += `- \`/${commandName}\` -> \`.claude/commands/${commandName}.md\`\n`;
    }
    content += '\n';
  }

  content += '## Skills\n\n';
  content += 'Codex discovers skills from `.agents/skills/*/SKILL.md`.\n';
  content += 'Each generated skill file is a minimal wrapper around the canonical `.claude/skills/*/SKILL.md` source.\n\n';
  for (const skillDir of skillDirs) {
    content += `- \`${skillDir}\` -> \`.claude/skills/${skillDir}/SKILL.md\`\n`;
  }
  content += '\n';

  content += '## Agent Roles\n\n';
  if (agentNames.length === 0) {
    content += 'No specialized agent role playbooks are currently defined.\n';
  } else {
    for (const agentName of agentNames) {
      content += `- \`${agentName}\` -> \`.claude/agents/${agentName}.md\`\n`;
    }
  }
  content += '\n';

  content += '## Generated File Policy\n\n';
  content += '- `AGENTS.md`, `.agents/skills/*`, `GEMINI.md`, `.gemini/*`, and `.github/copilot-instructions.md` are generated from `.claude/`.\n';
  content += '- Do not edit generated files by hand. Update `CLAUDE.md` or `.claude/**`, then rerun `pnpm ai:generate`.\n';

  return content;
}

async function generateAgentsMd(skillDirs: string[], agentNames: string[], commandNames: string[]) {
  return renderGenerated(
    'AGENTS.md',
    renderAgentsMarkdown({
      skillDirs,
      agentNames,
      commandNames,
    }),
  );
}

async function generateCodexSkillWrappers(skillDirs: string[]) {
  const files: GeneratedFile[] = [];

  for (const skillDir of skillDirs) {
    const canonical = read(`.claude/skills/${skillDir}/SKILL.md`);
    const { name, description } = parseSkillFrontmatter(canonical);
    files.push(
      await renderGenerated(
        `.agents/skills/${skillDir}/SKILL.md`,
        renderCodexSkillWrapper({ skillDir, name, description }),
      ),
    );
  }

  let readme = '# Codex Skills (Auto-generated Wrappers)\n\n';
  readme += 'Each generated skill wraps the canonical `.claude/skills/*/SKILL.md` file.\n';
  readme += 'Do not edit files in `.agents/skills/` by hand.\n\n';
  readme += '| Skill | Canonical Source |\n';
  readme += '| --- | --- |\n';
  for (const skillDir of skillDirs) {
    readme += `| ${skillDir} | \`.claude/skills/${skillDir}/SKILL.md\` |\n`;
  }

  files.push(await renderGenerated('.agents/skills/README.md', readme));

  return files;
}

async function generateGemini(ruleBasenames: string[], claudeMd: string) {
  let gemini = '# Alder Grove — Gemini Project Instructions\n\n';
  gemini += 'This file is auto-generated from `.claude/`. Update the canonical sources there instead.\n\n';
  gemini += '## Style Guide\n\n';
  gemini += '@.gemini/styleguide.md\n\n';
  gemini += '## Domain Rules\n\n';
  for (const ruleName of ruleBasenames) {
    gemini += `@.claude/rules/${ruleName}.md\n`;
  }
  gemini += '\n';

  let styleguide = '# Alder Grove Coding Standards\n\n';
  styleguide += 'Extracted from the canonical `CLAUDE.md`.\n\n';
  for (const heading of STYLEGUIDE_SECTIONS) {
    const section = extractSection(claudeMd, heading);
    if (!section) {
      continue;
    }
    styleguide += `${section}\n\n---\n\n`;
  }
  styleguide = styleguide.replace(/\n---\n\n$/, '\n');

  return [
    await renderGenerated('GEMINI.md', gemini),
    await renderGenerated('.gemini/styleguide.md', styleguide),
  ];
}

async function generateCopilot(ruleBasenames: string[], claudeMd: string) {
  let repoInstructions = '# Alder Grove — GitHub Copilot Instructions\n\n';
  repoInstructions += 'These instructions are auto-generated from `CLAUDE.md` and `.claude/rules/*.md`.\n';
  repoInstructions += 'Do not edit this file by hand.\n\n';

  for (const heading of STYLEGUIDE_SECTIONS) {
    const section = extractSection(claudeMd, heading);
    if (!section) {
      continue;
    }
    repoInstructions += `${section}\n\n---\n\n`;
  }
  repoInstructions = repoInstructions.replace(/\n---\n\n$/, '\n');

  const files: GeneratedFile[] = [await renderGenerated('.github/copilot-instructions.md', repoInstructions)];

  for (const ruleName of ruleBasenames) {
    const canonical = read(`.claude/rules/${ruleName}.md`);
    const applyTo = parseRulePaths(canonical).join(',');
    const body = stripFrontmatter(canonical).trimEnd();
    const content = `---
applyTo: "${applyTo}"
---

${body}
`;

    files.push(await renderGenerated(`.github/instructions/${ruleName}.instructions.md`, content));
  }

  return files;
}

async function buildGeneratedFiles(): Promise<GeneratedFile[]> {
  const claudeMd = read('CLAUDE.md');
  const skillDirs = listDirectories('.claude/skills', 'SKILL.md');
  const agentNames = listMarkdownBasenames('.claude/agents');
  const commandNames = listMarkdownBasenames('.claude/commands');
  const ruleBasenames = listMarkdownBasenames('.claude/rules');

  return [
    await generateAgentsMd(skillDirs, agentNames, commandNames),
    ...(await generateCodexSkillWrappers(skillDirs)),
    ...(await generateGemini(ruleBasenames, claudeMd)),
    ...(await generateCopilot(ruleBasenames, claudeMd)),
  ];
}

function assertFilesAreCurrent(files: GeneratedFile[]): void {
  const staleFiles = files
    .filter(
      (file) =>
        !existsSync(join(ROOT, file.path)) ||
        readFileSync(join(ROOT, file.path), 'utf8') !== file.content,
    )
    .map((file) => file.path);

  if (staleFiles.length > 0) {
    throw new Error(`Stale AI config files:\n${staleFiles.join('\n')}\n\nRun: pnpm ai:generate`);
  }

  const skillDirs = listDirectories('.claude/skills', 'SKILL.md');
  const ruleBasenames = listMarkdownBasenames('.claude/rules');
  const expectedPaths = new Set(generatedFilePaths(skillDirs, ruleBasenames));

  if (existsSync(join(ROOT, '.agents/agents'))) {
    throw new Error('Legacy `.agents/agents` directory is stale. Remove it and rerun `pnpm ai:generate`.');
  }

  const generatedSkillRoot = join(ROOT, '.agents/skills');
  if (existsSync(generatedSkillRoot)) {
    const skillEntries = readdirSync(generatedSkillRoot, { withFileTypes: true })
      .filter((entry: Dirent) => entry.isDirectory())
      .map((entry: Dirent) => `.agents/skills/${entry.name}/SKILL.md`);

    for (const skillPath of skillEntries) {
      if (!expectedPaths.has(skillPath)) {
        throw new Error(`Unexpected generated skill wrapper present: ${skillPath}`);
      }
    }
  }
}

async function run(): Promise<void> {
  const files = await buildGeneratedFiles();

  if (CHECK_MODE) {
    assertFilesAreCurrent(files);
    return;
  }

  removeStaleGeneratedDirectories();
  writeGeneratedFiles(files);
  stageGeneratedFiles(files);
}

function isMainModule(): boolean {
  const entry = process.argv[1];
  if (!entry) {
    return false;
  }

  return resolve(entry) === fileURLToPath(import.meta.url);
}

if (isMainModule()) {
  run().catch((error) => {
    const message = error instanceof Error ? error.message : String(error);
    console.error(message);
    process.exitCode = 1;
  });
}

export { run };
