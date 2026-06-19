import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import type { ApiErrorDto, ErrorResponseDto, FilterDto, ParseResponseDto } from './parse-contract'

let lastSelectedDirectory: string | null = null

export async function selectJsonlPath(): Promise<string | null> {
  const defaultPath = await invoke<string>('resolve_jsonl_initial_directory', {
    rememberedDirectory: lastSelectedDirectory,
  })
  const selected = await open({
    defaultPath,
    multiple: false,
    directory: false,
    filters: [
      {
        name: 'Codex JSONL',
        extensions: ['jsonl'],
      },
    ],
  })

  if (typeof selected !== 'string') {
    return null
  }

  lastSelectedDirectory = parentDirectory(selected)
  return selected
}

export async function parseSelectedJsonl(
  path: string,
  filter: FilterDto,
): Promise<ParseResponseDto> {
  try {
    return await invoke<ParseResponseDto>('parse_selected_jsonl', { path, filter })
  } catch (error) {
    throw normalizeApiError(error)
  }
}

function normalizeApiError(error: unknown): ApiErrorDto {
  if (isErrorResponse(error)) {
    return error.error
  }

  if (error instanceof Error) {
    return {
      code: 'tauri_command_failed',
      message: error.message,
    }
  }

  return {
    code: 'tauri_command_failed',
    message: String(error),
  }
}

function isErrorResponse(error: unknown): error is ErrorResponseDto {
  const candidate = error as ErrorResponseDto

  return (
    typeof error === 'object' &&
    error !== null &&
    'error' in error &&
    typeof candidate.error === 'object' &&
    candidate.error !== null &&
    typeof candidate.error.code === 'string' &&
    typeof candidate.error.message === 'string'
  )
}

function parentDirectory(path: string): string | null {
  const normalized = path.replace(/[/\\]+$/, '')
  const lastSeparator = Math.max(normalized.lastIndexOf('/'), normalized.lastIndexOf('\\'))

  if (lastSeparator <= 0) {
    return null
  }

  return normalized.slice(0, lastSeparator)
}
