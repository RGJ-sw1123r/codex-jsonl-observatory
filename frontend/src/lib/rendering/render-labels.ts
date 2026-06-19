import type { EntryKind } from '../parse-contract'

export type RenderFamily = 'you' | 'codex' | 'meta' | 'tool' | 'result' | 'unknown'

export interface RenderLabel {
  family: RenderFamily
  label: string
  source_label: string
}

export function renderLabelForKind(kind: EntryKind | string, sourceLabel: string): RenderLabel {
  switch (kind) {
    case 'you':
      return { family: 'you', label: 'YOU', source_label: sourceLabel || '[YOU]' }
    case 'codex':
      return { family: 'codex', label: 'CODEX', source_label: sourceLabel || '[CODEX]' }
    case 'system':
      return { family: 'meta', label: 'SYSTEM', source_label: sourceLabel || '[SYSTEM]' }
    case 'context':
      return { family: 'meta', label: 'CONTEXT', source_label: sourceLabel || '[CONTEXT]' }
    case 'task':
      return { family: 'meta', label: 'META', source_label: sourceLabel || '[TASK]' }
    case 'tool_call':
      return { family: 'tool', label: 'TOOL', source_label: sourceLabel || '[TOOL CALL]' }
    case 'tool_result':
      return { family: 'result', label: 'RESULT', source_label: sourceLabel || '[TOOL RESULT]' }
    default:
      return { family: 'unknown', label: 'UNKNOWN', source_label: sourceLabel || '[UNKNOWN]' }
  }
}
