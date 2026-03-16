/**
 * Shared domain types — re-exports generated types + runtime helpers.
 *
 * Generated types come from grove-domain via ts-rs.
 * Regenerate with: pnpm generate:types
 */
export type { AiProvenance } from '../../../generated';

/** Type predicate narrowing entities to AI-authored. */
export function isAiAuthored(
  entity: { ai_authored: boolean },
): entity is { ai_authored: true } {
  return entity.ai_authored;
}
