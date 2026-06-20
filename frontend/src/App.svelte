<script lang="ts">
  import {
    applyParseResponse,
    beginLoad,
    createInitialLoadWorkflowState,
    defaultFilterState,
    failLoad,
    selectPath,
    updateFilter,
    type LoadWorkflowState,
  } from './lib/load-workflow'
  import {
    exportWorklog,
    parseSelectedJsonl,
    selectJsonlPath,
    selectWorklogParentDirectory,
  } from './lib/tauri-bridge'
  import RenderedEntry from './lib/rendering/RenderedEntry.svelte'
  import ChatTranscript from './lib/rendering/ChatTranscript.svelte'
  import MarkdownTranscript from './lib/rendering/MarkdownTranscript.svelte'
  import TerminalTranscript from './lib/rendering/TerminalTranscript.svelte'
  import {
    renderPathForTheme,
    transcriptThemes,
    type TranscriptThemeName,
  } from './lib/rendering/transcript-themes'
  import type { ApiErrorDto, LoadedFileMetadataDto } from './lib/parse-contract'

  let workflow: LoadWorkflowState = createInitialLoadWorkflowState()
  let actionStatusMessage = ''
  let isExportingWorklog = false
  let rawEntriesPage = 1
  let selectedTheme: TranscriptThemeName = 'Terminal Style'

  const RAW_ENTRIES_PAGE_SIZE = 50

  const filterOptions = [
    ['show_you', 'You'],
    ['show_codex', 'Codex'],
    ['show_tool_call', 'Tool calls'],
    ['show_tool_result', 'Tool results'],
    ['show_meta', 'Meta'],
  ] as const

  function handlePathInput(event: Event) {
    const target = event.currentTarget as HTMLInputElement
    workflow = selectPath(workflow, target.value)
    actionStatusMessage = ''
    rawEntriesPage = 1
  }

  async function chooseJsonlPath() {
    const selectedPath = await selectJsonlPath()

    if (selectedPath !== null) {
      workflow = selectPath(workflow, selectedPath)
      actionStatusMessage = ''
      rawEntriesPage = 1
      await loadSelectedJsonl(selectedPath)
    }
  }

  async function loadSelectedJsonl(pathOverride?: string) {
    const path = (pathOverride ?? workflow.selected_file.path).trim()

    if (path === '') {
      return
    }

    workflow = beginLoad(workflow)
    rawEntriesPage = 1

    try {
      const response = await parseSelectedJsonl(path, defaultFilterState)
      workflow = applyParseResponse(workflow, response)
      actionStatusMessage = ''
    } catch (error) {
      workflow = failLoad(workflow, normalizeLoadError(error))
      actionStatusMessage = ''
    }
  }

  function handleFilterInput(key: keyof LoadWorkflowState['filter'], event: Event) {
    const target = event.currentTarget as HTMLInputElement
    workflow = updateFilter(workflow, key, target.checked)
    rawEntriesPage = 1
  }

  function handleThemeInput(event: Event) {
    const target = event.currentTarget as HTMLSelectElement
    selectedTheme = target.value as TranscriptThemeName
  }

  function rawEntriesPageCount() {
    return Math.max(1, Math.ceil(workflow.observations.entries.length / RAW_ENTRIES_PAGE_SIZE))
  }

  function rawEntriesPageStartIndex() {
    return (rawEntriesPage - 1) * RAW_ENTRIES_PAGE_SIZE
  }

  function paginatedRawEntries() {
    const start = rawEntriesPageStartIndex()
    return workflow.observations.entries.slice(start, start + RAW_ENTRIES_PAGE_SIZE)
  }

  function rawEntriesShowingStart() {
    return workflow.observations.entries.length === 0 ? 0 : rawEntriesPageStartIndex() + 1
  }

  function rawEntriesShowingEnd() {
    return Math.min(
      rawEntriesPageStartIndex() + RAW_ENTRIES_PAGE_SIZE,
      workflow.observations.entries.length,
    )
  }

  function showPreviousRawEntriesPage() {
    rawEntriesPage = Math.max(1, rawEntriesPage - 1)
  }

  function showNextRawEntriesPage() {
    rawEntriesPage = Math.min(rawEntriesPageCount(), rawEntriesPage + 1)
  }

  function displayedAbsolutePath() {
    const path = workflow.loaded_file.metadata?.absolute_path ?? workflow.selected_file.path
    return path === '' ? 'None' : displayFriendlyPath(path)
  }

  function displayedMetadata(): LoadedFileMetadataDto | null {
    const metadata = workflow.loaded_file.metadata

    if (metadata === null) {
      return null
    }

    return {
      ...metadata,
      absolute_path: displayFriendlyPath(metadata.absolute_path),
    }
  }

  function displayFriendlyPath(path: string) {
    return path.replace(/^\\\\\?\\UNC\\/i, '\\\\').replace(/^\\\\\?\\/, '')
  }

  function transcriptKey() {
    const filter = workflow.filter
    return [
      workflow.loaded_file.metadata?.absolute_path ?? 'unloaded',
      filter.show_you,
      filter.show_codex,
      filter.show_tool_call,
      filter.show_tool_result,
      filter.show_meta,
      selectedTheme,
    ].join('|')
  }

  async function copyResumeCommand() {
    const command = workflow.loaded_file.metadata?.resume_command

    if (command == null || command.trim() === '') {
      return
    }

    try {
      if (navigator.clipboard == null) {
        throw new Error('Clipboard access is unavailable.')
      }

      await navigator.clipboard.writeText(command)
      actionStatusMessage = 'Copied.'
    } catch {
      actionStatusMessage = 'Copy failed.'
    }
  }

  async function handleExportWorklog() {
    const metadata = workflow.loaded_file.metadata

    if (workflow.status !== 'loaded' || metadata === null || isExportingWorklog) {
      return
    }

    isExportingWorklog = true

    try {
      const parentDirectory = await selectWorklogParentDirectory(metadata.absolute_path)
      if (parentDirectory === null) {
        actionStatusMessage = 'Export cancelled.'
        return
      }

      actionStatusMessage = 'Exporting worklog…'
      const result = await exportWorklog(metadata.absolute_path, parentDirectory)
      if (!result.folder_opened) {
        actionStatusMessage = result.refreshed
          ? 'Worklog refreshed, but folder could not be opened.'
          : 'Worklog exported, but folder could not be opened.'
      } else {
        actionStatusMessage = result.refreshed
          ? `Worklog refreshed: ${displayFriendlyPath(result.bundle_path)}`
          : `Worklog exported: ${displayFriendlyPath(result.bundle_path)}`
      }
    } catch (error) {
      const apiError = normalizeLoadError(error)
      actionStatusMessage =
        apiError.code === 'target_not_safe_to_overwrite'
          ? 'Target not safe to overwrite.'
          : `Worklog export failed: ${apiError.message}`
    } finally {
      isExportingWorklog = false
    }
  }

  function hasSelectedPath() {
    return workflow.selected_file.path.trim() !== ''
  }

  function normalizeLoadError(error: unknown): ApiErrorDto {
    if (isApiError(error)) {
      return error
    }

    if (error instanceof Error) {
      return { code: 'tauri_command_failed', message: error.message }
    }

    return { code: 'tauri_command_failed', message: String(error) }
  }

  function isApiError(error: unknown): error is ApiErrorDto {
    return (
      typeof error === 'object' &&
      error !== null &&
      'code' in error &&
      'message' in error &&
      typeof (error as ApiErrorDto).code === 'string' &&
      typeof (error as ApiErrorDto).message === 'string'
    )
  }
</script>

<main class="control-room">
  <header class="app-header" aria-labelledby="app-title">
    <div class="toolbar">
      <div class="product-heading">
        <p class="eyebrow">Local transcript viewer</p>
        <h1 id="app-title">Codex JSONL Observatory</h1>
      </div>

      <div class="toolbar-actions">
        <button type="button" onclick={chooseJsonlPath}>Select JSONL</button>

        <label class="manual-path-field">
          <span>Manual path (use Refresh to load)</span>
          <input
            type="text"
            value={workflow.selected_file.path}
            placeholder="Paste a local JSONL path"
            oninput={handlePathInput}
          />
        </label>

        <button
          type="button"
          class="refresh-button"
          disabled={!hasSelectedPath()}
          onclick={() => loadSelectedJsonl()}
        >
          Refresh
        </button>
        <span class="status" data-status={workflow.status}>{workflow.status}</span>
      </div>
    </div>

    <section class="selection-panel" aria-label="Current selection">
      <div class="selected-path-row">
        <span>Selected path</span>
        <strong
          title={workflow.selected_file.path
            ? displayFriendlyPath(workflow.selected_file.path)
            : 'No JSONL path selected.'}
        >
          {workflow.selected_file.path
            ? displayFriendlyPath(workflow.selected_file.path)
            : 'No JSONL path selected.'}
        </strong>
      </div>

      <dl class="selection-metadata">
        <div>
          <dt>File</dt>
          <dd>{workflow.loaded_file.metadata?.file_name ?? 'Not loaded'}</dd>
        </div>
        <div>
          <dt>Session ID</dt>
          <dd>{workflow.loaded_file.metadata?.session_id ?? 'Not detected'}</dd>
        </div>
      </dl>

      <div class="resume-row">
        <span>Resume command</span>
        <code>{workflow.loaded_file.metadata?.resume_command ?? 'Not available'}</code>
        <div class="resume-actions">
          <button
            type="button"
            class="secondary resume-action-button"
            disabled={workflow.loaded_file.metadata?.resume_command == null}
            onclick={copyResumeCommand}
          >
            Copy Resume Command
          </button>
          <button
            type="button"
            class="secondary resume-action-button"
            disabled={workflow.status !== 'loaded' || isExportingWorklog}
            onclick={handleExportWorklog}
          >
            Export Worklog
          </button>
        </div>
        <span class="action-status" aria-live="polite">{actionStatusMessage}</span>
      </div>
    </section>

    <section class="filter-bar" aria-label="Transcript filters">
      <h2>Filters</h2>
      <div class="filter-list">
        {#each filterOptions as [key, label]}
          <label>
            <input
              type="checkbox"
              checked={workflow.filter[key]}
              onchange={(event) => handleFilterInput(key, event)}
            />
            <span>{label}</span>
          </label>
        {/each}
      </div>
      <label class="theme-selector">
        <span>Theme</span>
        <select value={selectedTheme} onchange={handleThemeInput}>
          {#each transcriptThemes as theme}
            <option value={theme}>{theme}</option>
          {/each}
        </select>
      </label>
    </section>

    {#if workflow.status === 'loading'}
      <p class="status-message" aria-live="polite">Loading the selected JSONL file…</p>
    {:else if workflow.status === 'error'}
      <p class="status-message error" aria-live="assertive">
        {workflow.error?.message ?? 'The selected JSONL file could not be loaded.'}
      </p>
    {/if}
  </header>

  <section class="terminal-section" aria-label="Transcript view">
    <article
      class:terminal-panel={renderPathForTheme(selectedTheme) === 'terminal'}
      class:theme-panel={renderPathForTheme(selectedTheme) !== 'terminal'}
    >
      {#key transcriptKey()}
        {#if renderPathForTheme(selectedTheme) === 'terminal'}
          <TerminalTranscript
            theme={selectedTheme}
            isLoaded={workflow.status === 'loaded'}
            showIdentityNote={workflow.status !== 'loaded'}
            metadata={displayedMetadata()}
            observedEventCounts={workflow.loaded_file.observed_event_counts}
            blocks={workflow.observations.transcript_blocks}
          />
        {:else if renderPathForTheme(selectedTheme) === 'markdown'}
          <MarkdownTranscript
            theme={selectedTheme}
            isLoaded={workflow.status === 'loaded'}
            metadata={displayedMetadata()}
            observedEventCounts={workflow.loaded_file.observed_event_counts}
            blocks={workflow.observations.transcript_blocks}
          />
        {:else}
          <ChatTranscript
            theme={selectedTheme as 'DM Style' | 'DM Style (Dark)'}
            isLoaded={workflow.status === 'loaded'}
            metadata={displayedMetadata()}
            observedEventCounts={workflow.loaded_file.observed_event_counts}
            blocks={workflow.observations.transcript_blocks}
          />
        {/if}
      {/key}
    </article>
  </section>

  <details class="secondary-section">
    <summary>
      <span>Raw Entries &amp; Diagnostics</span>
      <span class="count">{workflow.observations.entries.length}</span>
    </summary>

    <div class="secondary-content">
      <section aria-labelledby="raw-entries-title">
        <div class="raw-entries-heading">
          <h2 id="raw-entries-title">Raw Entries</h2>
          {#if workflow.observations.entries.length > 0}
            <div class="raw-entries-pagination" aria-label="Raw Entries pagination">
              <span>Page {rawEntriesPage} / {rawEntriesPageCount()}</span>
              <span>
                Showing {rawEntriesShowingStart()}–{rawEntriesShowingEnd()} of
                {workflow.observations.entries.length}
              </span>
              <button
                type="button"
                class="secondary pagination-button"
                disabled={rawEntriesPage === 1}
                onclick={showPreviousRawEntriesPage}
              >
                Previous
              </button>
              <button
                type="button"
                class="secondary pagination-button"
                disabled={rawEntriesPage === rawEntriesPageCount()}
                onclick={showNextRawEntriesPage}
              >
                Next
              </button>
            </div>
          {/if}
        </div>
        {#if workflow.observations.entries.length === 0}
          <p class="empty-text">No parsed entries to display.</p>
        {:else}
          <ol class="render-list">
            {#each paginatedRawEntries() as entry, index}
              <RenderedEntry {entry} index={rawEntriesPageStartIndex() + index} />
            {/each}
          </ol>
        {/if}
      </section>

      <aside class="diagnostics" aria-label="Parse diagnostics">
        <h2>Resolved Path</h2>
        <p class="diagnostic-path" title={displayedAbsolutePath()}>{displayedAbsolutePath()}</p>

        <h2 class="diagnostic-heading">Counters</h2>
        <dl class="diagnostic-metrics">
          <div><dt>Parsed</dt><dd>{workflow.loaded_file.counters.parsed_candidates}</dd></div>
          <div><dt>Total</dt><dd>{workflow.loaded_file.counters.total_entries}</dd></div>
          <div><dt>Visible</dt><dd>{workflow.loaded_file.counters.visible_entries}</dd></div>
          <div><dt>Ignored</dt><dd>{workflow.loaded_file.counters.ignored_lines}</dd></div>
          <div><dt>Malformed</dt><dd>{workflow.loaded_file.counters.malformed_lines}</dd></div>
        </dl>

        <h2 class="diagnostic-heading">Observed Events</h2>
        {#if workflow.loaded_file.observed_event_counts.length === 0}
          <p class="empty-text">No observed event counts loaded.</p>
        {:else}
          <ol class="event-list">
            {#each workflow.loaded_file.observed_event_counts as eventCount}
              <li>
                <span>{eventCount.event}</span>
                <strong>{eventCount.count}</strong>
              </li>
            {/each}
          </ol>
        {/if}
      </aside>
    </div>
  </details>
</main>
