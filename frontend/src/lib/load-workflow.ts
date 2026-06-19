import type {
  ApiErrorDto,
  FilterDto,
  LoadedFileMetadataDto,
  ObservedEventCountDto,
  ParseCountersDto,
  ParseResponseDto,
  RenderedEntryDto,
  TranscriptBlockDto,
} from './parse-contract'

export type LoadStatus = 'idle' | 'selected' | 'loading' | 'loaded' | 'error'

export interface SelectedBrowserFile {
  name: string
  size: number
  type: string
  last_modified: number
}

export interface SelectedFileState {
  path: string
  browser_file: SelectedBrowserFile | null
}

export interface LoadedFileState {
  metadata: LoadedFileMetadataDto | null
  counters: ParseCountersDto
  observed_event_counts: ObservedEventCountDto[]
}

export interface ParsedObservationState {
  entries: RenderedEntryDto[]
  transcript_blocks: TranscriptBlockDto[]
}

export interface LoadWorkflowState {
  status: LoadStatus
  selected_file: SelectedFileState
  loaded_file: LoadedFileState
  observations: ParsedObservationState
  filter: FilterDto
  error: ApiErrorDto | null
}

export const defaultFilterState: FilterDto = {
  show_you: true,
  show_codex: true,
  show_tool_call: true,
  show_tool_result: true,
  show_meta: true,
}

export const emptyCounters: ParseCountersDto = {
  parsed_candidates: 0,
  total_entries: 0,
  visible_entries: 0,
  ignored_lines: 0,
  malformed_lines: 0,
}

export function createInitialLoadWorkflowState(): LoadWorkflowState {
  return {
    status: 'idle',
    selected_file: {
      path: '',
      browser_file: null,
    },
    loaded_file: {
      metadata: null,
      counters: { ...emptyCounters },
      observed_event_counts: [],
    },
    observations: {
      entries: [],
      transcript_blocks: [],
    },
    filter: { ...defaultFilterState },
    error: null,
  }
}

export function selectPath(state: LoadWorkflowState, path: string): LoadWorkflowState {
  return {
    ...clearLoadedResult(state),
    status: path.trim() === '' ? 'idle' : 'selected',
    selected_file: {
      ...state.selected_file,
      path,
    },
  }
}

export function selectBrowserFile(state: LoadWorkflowState, file: File | null): LoadWorkflowState {
  return {
    ...clearLoadedResult(state),
    status: file === null && state.selected_file.path.trim() === '' ? 'idle' : 'selected',
    selected_file: {
      ...state.selected_file,
      browser_file:
        file === null
          ? null
          : {
              name: file.name,
              size: file.size,
              type: file.type,
              last_modified: file.lastModified,
            },
    },
  }
}

export function beginLoad(state: LoadWorkflowState): LoadWorkflowState {
  return {
    ...state,
    status: 'loading',
    error: null,
  }
}

export function applyParseResponse(
  state: LoadWorkflowState,
  response: ParseResponseDto,
): LoadWorkflowState {
  return {
    ...state,
    status: 'loaded',
    loaded_file: {
      metadata: response.source,
      counters: response.parsed_chat_log.counters,
      observed_event_counts: response.parsed_chat_log.observed_event_counts,
    },
    observations: {
      entries: response.parsed_chat_log.entries,
      transcript_blocks: response.parsed_chat_log.transcript_blocks,
    },
    error: null,
  }
}

export function failLoad(state: LoadWorkflowState, error: ApiErrorDto): LoadWorkflowState {
  return {
    ...clearLoadedResult(state),
    status: 'error',
    error,
  }
}

export function updateFilter(
  state: LoadWorkflowState,
  key: keyof FilterDto,
  value: boolean,
): LoadWorkflowState {
  return {
    ...state,
    filter: {
      ...state.filter,
      [key]: value,
    },
  }
}

function clearLoadedResult(state: LoadWorkflowState): LoadWorkflowState {
  return {
    ...state,
    loaded_file: {
      metadata: null,
      counters: { ...emptyCounters },
      observed_event_counts: [],
    },
    observations: {
      entries: [],
      transcript_blocks: [],
    },
    error: null,
  }
}
