import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import type { ApiErrorDto, ErrorResponseDto, FilterDto, ParseResponseDto } from './parse-contract'

export async function selectJsonlPath(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: 'Codex JSONL',
        extensions: ['jsonl'],
      },
    ],
  })

  return typeof selected === 'string' ? selected : null
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
