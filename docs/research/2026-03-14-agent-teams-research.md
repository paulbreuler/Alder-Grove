# Claude Code Agents & Agent Teams — 2026 Best Practices Research

> Research date: 2026-03-14

## Summary

Research into Claude Code agent teams, the Claude Agent SDK, custom subagents,
multi-agent orchestration patterns, and the OSS agent ecosystem to inform Alder
Grove's development tooling and ACP product design.

---

## 1. Claude Code Agent Teams

Launched February 2026 with Opus 4.6 as an experimental feature.

**Enable**: `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in settings or environment.

**Architecture**: Team lead orchestrates teammates via a shared task list and
mailbox messaging system. Teammates claim tasks, message each other directly,
and coordinate autonomously — distinct from subagents which only report back to
the parent.

| Concept        | Description                                                   |
| -------------- | ------------------------------------------------------------- |
| Team lead      | Orchestrates work, creates task list, assigns initial tasks   |
| Teammates      | Independent Claude Code sessions that claim tasks             |
| Shared tasks   | Dependency tracking + file-locking for claims                 |
| Mailbox        | Direct messaging between teammates (`write` > `broadcast`)   |
| Config         | `~/.claude/teams/{name}/config.json`                          |
| Tasks          | `~/.claude/tasks/{name}/`                                     |

### Best Practices

- **3-5 teammates** — sweet spot for coordination overhead vs throughput
- **5-6 tasks per teammate** — enough to stay busy, not overwhelming
- **Avoid file conflicts** — partition work so teammates don't edit the same files
- **Worktree isolation** — each teammate gets its own git worktree
- **Plan approval** for risky/architectural work
- **Start with read-only research** before parallel implementation
- **Quality gate hooks**: `TeammateIdle` and `TaskCompleted` hooks enforce tests/lint

### Docs

- https://code.claude.com/docs/en/agent-teams
- https://claudefa.st/blog/guide/agents/agent-teams
- https://addyosmani.com/blog/claude-code-agent-teams/

---

## 2. Claude Agent SDK

Formerly "Claude Code SDK", renamed to **Claude Agent SDK**.

- Python: `pip install claude-agent-sdk`
- TypeScript: `npm install @anthropic-ai/claude-agent-sdk`

**Key difference from Anthropic Client SDK**: The Client SDK requires you to
implement the tool loop. The Agent SDK handles tool execution autonomously with
built-in tools (Read, Write, Edit, Bash, Glob, Grep, WebSearch, WebFetch).

Supports: hooks, subagents, MCP servers, permissions, session management
(resume/fork), skills, and memory.

### Docs

- https://github.com/anthropics/claude-agent-sdk-typescript
- https://github.com/anthropics/claude-agent-sdk-python
- https://platform.claude.com/docs/en/agent-sdk/overview

---

## 3. Custom Subagents

Defined as Markdown + YAML frontmatter in `.claude/agents/` (project) or
`~/.claude/agents/` (user).

### Frontmatter Options

```yaml
name: domain-expert
description: Reviews domain layer changes
tools: [Read, Grep, Glob, Bash]          # whitelist
disallowedTools: [Edit, Write]            # blacklist
model: opus                               # or sonnet, haiku
permissionMode: bypassPermissions         # or default, plan, auto
maxTurns: 25
skills: [check-architecture]              # preload skills
mcpServers: [pencil]                      # MCP servers
hooks: { ... }                            # lifecycle hooks
memory: project                           # user|project|local
background: true                          # run in background
isolation: worktree                       # git worktree isolation
```

### Built-in Agent Types

| Type             | Model     | Capabilities                     |
| ---------------- | --------- | -------------------------------- |
| `Explore`        | Haiku     | Read-only, fast exploration      |
| `Plan`           | Inherited | Research for plan mode           |
| `general-purpose`| Inherited | Full tool access                 |

### Docs

- https://code.claude.com/docs/en/sub-agents
- https://github.com/VoltAgent/awesome-claude-code-subagents

---

## 4. AGENTS.md Standard

Now governed by the **Linux Foundation's Agentic AI Foundation (AAIF)**. Cross-tool
standard read by Codex CLI, GitHub Copilot, Cursor, Windsurf, Amp, and Devin.

- Coexists with CLAUDE.md — AGENTS.md for universal, CLAUDE.md for Claude-specific
- Best practice: ~150-200 instructions max, be concrete, iterate
- Alder Grove already generates AGENTS.md from `.claude/` sources

### Docs

- https://github.com/agentsmd/agents.md

---

## 5. Six Orchestration Patterns

| Pattern                  | When to Use                              | Alder Grove Fit                       |
| ------------------------ | ---------------------------------------- | ------------------------------------- |
| Parallel Specialists     | Multiple reviewers with different lenses | Security + architecture + test review |
| Pipeline                 | Sequential with dependencies             | Chunk 0 → Chunks 1-4                 |
| Self-Organizing Swarm    | Workers claim from shared task list      | Agent teams with task board           |
| Research → Implementation| Explore before coding                    | Brainstorm → plan → execute           |
| Plan Approval            | Architecture-boundary changes            | Plan mode + gate definitions          |
| Competing Hypotheses     | Adversarial debugging                    | Bug triage                            |

---

## 6. Top 8 Multi-Agent Patterns (2026 Consensus)

1. **Hierarchical Delegation** — parent delegates to scoped child agents
2. **DAG-Based Parallel Execution** — dependency graph; independent tasks in parallel
3. **Agent-as-Tool** — agents invokable as tools by other agents
4. **Role-Based Teams** — agents assigned roles (reviewer, architect, tester)
5. **Event-Driven Flows** — decorator/state-machine triggered activation
6. **Graph-Based Workflows** — nodes + edges + state transforms
7. **Evaluate-Fix Loops** — bounded auto-fix iterations on failure
8. **Progressive Context Disclosure** — token-efficient lazy context loading

---

## 7. OSS Agent Frameworks

### General-Purpose

| Framework            | Stars | Key Innovation                                   |
| -------------------- | ----- | ------------------------------------------------ |
| AutoGen (Microsoft)  | 55.6k | Most mature multi-agent; 3-layer API; Studio GUI |
| CrewAI               | Large | Crews (autonomy) + Flows (control); event-driven |
| LangGraph            | Large | Graph orchestration; durable execution            |
| OpenAI Agents SDK    | —     | Handoffs + Guardrails; Realtime Agents (voice)   |
| Google ADK           | —     | A2A protocol; session rewinding                  |
| Pydantic AI          | —     | Type-safe; dependency injection                  |
| smolagents (HF)      | —     | "Code as action" (30% fewer steps); ~1000 lines  |

### Coding-Specific

| Tool             | Key Innovation                            | SWE-Bench |
| ---------------- | ----------------------------------------- | --------- |
| OpenHands        | Highest benchmark; SDK+CLI+GUI+Cloud      | 77.6%     |
| OpenAI Codex CLI | Rust core; shell-tool-MCP                 | —         |
| SWE-agent        | Research-focused; maximum LM agency       | 70.8%     |
| Cline            | VS Code extension; AST-aware context      | —         |
| Goose (Block)    | Rust core; multi-model cost optimization  | —         |

### Rust Agent Libraries

| Library  | Key Feature                                                    |
| -------- | -------------------------------------------------------------- |
| Rig      | Most mature Rust agent lib. Builder pattern, 20+ providers, WASM |
| Swiftide | Indexing + agentic pipelines. Tree-sitter, PostgreSQL, OTel    |

### Protocols

| Protocol | Description                                                        |
| -------- | ------------------------------------------------------------------ |
| MCP      | De facto standard for tool interop. Universal support.             |
| A2A      | Google JSON-RPC 2.0 inter-agent protocol. Maps closely to ACP.    |

---

## 8. Superpowers Ecosystem

| Project                | Stars | Key Feature                                          |
| ---------------------- | ----- | ---------------------------------------------------- |
| Superpowers Marketplace| 645   | Curated plugin registry for Claude Code              |
| SupaConductor          | 275   | 16 agents, 4-phase DAG dispatch, 42 skills           |

### SupaConductor Patterns Worth Adopting

- 4-Phase Workflow: Plan → Execute → Evaluate → Fix
- DAG-based parallel dispatch
- "Board of Directors" quality gate deliberation
- Evaluate-Fix loops with bounded iterations

### Docs

- https://github.com/obra/superpowers-marketplace
- https://github.com/Ibrahim-3d/conductor-orchestrator-superpowers

---

## 9. Relevance to Alder Grove

### For Development Tooling

- Agent teams map directly to chunked plan structure (Chunk 0 → Chunks 1-4)
- Role-based agents (domain-expert, api-developer, frontend-dev, architect)
  align with hexagonal architecture layers
- Worktree isolation prevents file conflicts during parallel work
- Quality gate hooks (`TaskCompleted`) enforce TDD and architecture compliance

### For ACP Product Design

- **Google A2A protocol** maps closely to ACP — worth reviewing for wire format
  compatibility and ecosystem interop
- **Agent Cards** (A2A) ≈ Agent entities in Grove
- **Guardrails** pattern appears in multiple frameworks (OpenAI Agents SDK,
  SupaConductor) — validates the product approach
- **Rig** and **Swiftide** are Rust-native agent libraries worth evaluating for
  the agent runtime in `grove-api`
