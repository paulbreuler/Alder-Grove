/**
 * Shared domain types — re-exports generated types + runtime helpers.
 *
 * Generated types come from grove-domain via ts-rs.
 * Regenerate with: pnpm generate:types
 */
export type { AiProvenance } from '../../../generated';

/** Type guard for AI-authored entities. */
export function isAiAuthored(entity: { ai_authored: boolean }): boolean {
  return entity.ai_authored;
}
