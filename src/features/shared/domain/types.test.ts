import { describe, it, expect } from 'vitest';
import { isAiAuthored } from './types';
import type {
  AiProvenance,
  Workspace,
  Session,
  Persona,
  Guardrail,
  GuardrailRule,
} from '../../../generated';

describe('isAiAuthored', () => {
  it('returns true for AI-authored content', () => {
    expect(isAiAuthored({ ai_authored: true })).toBe(true);
  });

  it('returns false for human-authored content', () => {
    expect(isAiAuthored({ ai_authored: false })).toBe(false);
  });
});

describe('Generated types', () => {
  it('AiProvenance uses null for optional fields', () => {
    const prov: AiProvenance = {
      ai_authored: false,
      ai_confidence: null,
      ai_rationale: null,
    };
    expect(prov.ai_authored).toBe(false);
    expect(prov.ai_confidence).toBeNull();
  });

  it('Workspace has required fields', () => {
    const ws: Workspace = {
      id: '00000000-0000-0000-0000-000000000000',
      org_id: 'org_1',
      name: 'Test',
      description: null,
      created_at: '2026-03-14T00:00:00Z',
      updated_at: '2026-03-14T00:00:00Z',
    };
    expect(ws.name).toBe('Test');
    expect(ws.description).toBeNull();
  });

  it('Persona flattens AiProvenance fields', () => {
    const p: Persona = {
      id: '00000000-0000-0000-0000-000000000000',
      workspace_id: '00000000-0000-0000-0000-000000000000',
      name: 'Developer',
      description: null,
      goals: null,
      pain_points: null,
      created_at: '2026-03-14T00:00:00Z',
      updated_at: '2026-03-14T00:00:00Z',
      ai_authored: true,
      ai_confidence: 0.95,
      ai_rationale: 'Generated',
    };
    expect(p.ai_authored).toBe(true);
    expect(p.ai_confidence).toBe(0.95);
  });

  it('Session has status and intent string unions', () => {
    const s: Session = {
      id: '00000000-0000-0000-0000-000000000000',
      workspace_id: '00000000-0000-0000-0000-000000000000',
      agent_id: '00000000-0000-0000-0000-000000000000',
      status: 'pending',
      intent: 'implement',
      target_type: null,
      target_id: null,
      config: null,
      created_at: '2026-03-14T00:00:00Z',
      updated_at: '2026-03-14T00:00:00Z',
    };
    expect(s.status).toBe('pending');
    expect(s.intent).toBe('implement');
  });

  it('GuardrailRule is a tagged union', () => {
    const rule: GuardrailRule = {
      type: 'prohibition',
      description: 'No deletions',
      patterns: ['*.prod.*'],
      actions: ['delete'],
    };
    expect(rule.type).toBe('prohibition');

    const g: Guardrail = {
      id: '00000000-0000-0000-0000-000000000000',
      workspace_id: '00000000-0000-0000-0000-000000000000',
      name: 'No deletions',
      description: null,
      category: 'prohibition',
      scope: 'workspace',
      enforcement: 'enforced',
      rule,
      version: 1,
      sort_order: 0,
      enabled: true,
      created_at: '2026-03-14T00:00:00Z',
      updated_at: '2026-03-14T00:00:00Z',
    };
    expect(g.enabled).toBe(true);
  });
});
