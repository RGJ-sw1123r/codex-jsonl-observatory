export const transcriptThemes = [
  'Terminal Style',
  'Markdown Style',
  'DM Style',
  'DM Style (Dark)',
] as const

export type TranscriptThemeName = (typeof transcriptThemes)[number]

export type TranscriptRenderPath = 'terminal' | 'markdown' | 'chat'

export function renderPathForTheme(theme: TranscriptThemeName): TranscriptRenderPath {
  switch (theme) {
    case 'Markdown Style':
      return 'markdown'
    case 'DM Style':
    case 'DM Style (Dark)':
      return 'chat'
    default:
      return 'terminal'
  }
}
