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

export interface SelectedFileState {
  path: string
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
  all_observations: ParsedObservationState
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
    },
    loaded_file: {
      metadata: null,
      counters: { ...emptyCounters },
      observed_event_counts: [],
    },
    all_observations: emptyObservations(),
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
      path,
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
  const allObservations = {
    entries: response.parsed_chat_log.entries,
    transcript_blocks: response.parsed_chat_log.transcript_blocks,
  }
  const observations = projectObservations(allObservations, state.filter)

  return {
    ...state,
    status: 'loaded',
    loaded_file: {
      metadata: response.source,
      counters: {
        ...response.parsed_chat_log.counters,
        visible_entries: observations.entries.length,
      },
      observed_event_counts: response.parsed_chat_log.observed_event_counts,
    },
    all_observations: allObservations,
    observations,
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
  const filter = {
    ...state.filter,
    [key]: value,
  }
  const observations = projectObservations(state.all_observations, filter)

  return {
    ...state,
    loaded_file: {
      ...state.loaded_file,
      counters: {
        ...state.loaded_file.counters,
        visible_entries: observations.entries.length,
      },
    },
    observations,
    filter,
  }
}

function projectObservations(
  observations: ParsedObservationState,
  filter: FilterDto,
): ParsedObservationState {
  return {
    entries: observations.entries.filter((entry) => filterAllowsKind(entry.kind, filter)),
    transcript_blocks: observations.transcript_blocks.filter((block) =>
      filterAllowsKind(block.entry_type, filter),
    ),
  }
}

function filterAllowsKind(kind: RenderedEntryDto['kind'], filter: FilterDto): boolean {
  switch (kind) {
    case 'you':
      return filter.show_you
    case 'codex':
      return filter.show_codex
    case 'tool_call':
      return filter.show_tool_call
    case 'tool_result':
      return filter.show_tool_result
    case 'context':
    case 'task':
    case 'system':
      return filter.show_meta
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
    all_observations: emptyObservations(),
    observations: emptyObservations(),
    error: null,
  }
}

function emptyObservations(): ParsedObservationState {
  return {
    entries: [],
    transcript_blocks: [],
  }
}
