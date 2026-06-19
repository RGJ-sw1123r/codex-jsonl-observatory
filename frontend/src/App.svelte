<script lang="ts">
  import {
    beginLoad,
    createInitialLoadWorkflowState,
    selectBrowserFile,
    selectPath,
    updateFilter,
    type LoadWorkflowState,
  } from './lib/load-workflow'

  let workflow: LoadWorkflowState = createInitialLoadWorkflowState()

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
  }

  function handleFileInput(event: Event) {
    const target = event.currentTarget as HTMLInputElement
    workflow = selectBrowserFile(workflow, target.files?.[0] ?? null)
  }

  function markLoadBoundary() {
    workflow = beginLoad(workflow)
  }

  function handleFilterInput(key: keyof LoadWorkflowState['filter'], event: Event) {
    const target = event.currentTarget as HTMLInputElement
    workflow = updateFilter(workflow, key, target.checked)
  }

  function fileSizeLabel(size: number) {
    if (size === 0) {
      return '0 B'
    }

    if (size < 1024) {
      return `${size} B`
    }

    if (size < 1024 * 1024) {
      return `${(size / 1024).toFixed(1)} KB`
    }

    return `${(size / (1024 * 1024)).toFixed(1)} MB`
  }

  function previewContent(content: string) {
    const compact = content.replace(/\s+/g, ' ').trim()
    return compact.length > 180 ? `${compact.slice(0, 180)}...` : compact
  }

  function displayedAbsolutePath() {
    return workflow.loaded_file.metadata?.absolute_path ?? (workflow.selected_file.path || 'None')
  }
</script>

<main class="control-room">
  <section class="load-panel" aria-labelledby="load-title">
    <div>
      <p class="eyebrow">Codex JSONL Observatory</p>
      <h1 id="load-title">Display Workflow Scaffold</h1>
    </div>

    <label class="field">
      <span>Signal record path</span>
      <input
        type="text"
        value={workflow.selected_file.path}
        placeholder="Select or enter a local JSONL path"
        oninput={handlePathInput}
      />
    </label>

    <label class="field">
      <span>Selected file</span>
      <input type="file" accept=".jsonl,application/jsonl,application/json" onchange={handleFileInput} />
    </label>

    <div class="actions">
      <button type="button" disabled={workflow.status === 'idle'} onclick={markLoadBoundary}>
        Mark load boundary
      </button>
      <span class="status" data-status={workflow.status}>{workflow.status}</span>
    </div>
  </section>

  {#if workflow.status === 'idle'}
    <section class="state-banner" aria-live="polite">
      <h2>Idle</h2>
      <p>No signal record selected.</p>
    </section>
  {:else if workflow.status === 'selected'}
    <section class="state-banner" aria-live="polite">
      <h2>Ready</h2>
      <p>A signal record is selected. The transport boundary is intentionally not implemented here.</p>
    </section>
  {:else if workflow.status === 'loading'}
    <section class="state-banner" aria-live="polite">
      <h2>Loading</h2>
      <p>Load state is active. Parsed observations will populate this scaffold after a future transport boundary.</p>
    </section>
  {:else if workflow.status === 'error'}
    <section class="state-banner error" aria-live="polite">
      <h2>Error</h2>
      <p>{workflow.error?.message ?? 'No error payload is available.'}</p>
    </section>
  {:else if workflow.status === 'loaded' && workflow.observations.entries.length === 0}
    <section class="state-banner" aria-live="polite">
      <h2>Loaded Empty</h2>
      <p>The selected signal record loaded without visible parsed entries.</p>
    </section>
  {:else if workflow.status === 'loaded'}
    <section class="state-banner" aria-live="polite">
      <h2>Loaded</h2>
      <p>Parsed entries and transcript blocks are available for display scaffolding.</p>
    </section>
  {/if}

  <section class="summary-grid" aria-label="Loaded file summary">
    <article class="metadata-panel">
      <div class="panel-heading">
        <h2>Loaded File</h2>
        <button
          type="button"
          class="secondary"
          disabled={workflow.loaded_file.metadata?.resume_command == null}
        >
          Copy Resume Command
        </button>
      </div>

      <dl>
        <div>
          <dt>File name</dt>
          <dd>{workflow.loaded_file.metadata?.file_name ?? 'Not loaded'}</dd>
        </div>
        <div>
          <dt>Absolute path</dt>
          <dd>{displayedAbsolutePath()}</dd>
        </div>
        <div>
          <dt>Session</dt>
          <dd>{workflow.loaded_file.metadata?.session_id ?? 'None'}</dd>
        </div>
        <div>
          <dt>Resume command</dt>
          <dd>{workflow.loaded_file.metadata?.resume_command ?? 'Unavailable'}</dd>
        </div>
      </dl>
    </article>

    <article>
      <h2>Selected File</h2>
      <dl>
        <div>
          <dt>Path input</dt>
          <dd>{workflow.selected_file.path || 'None'}</dd>
        </div>
        <div>
          <dt>Browser file</dt>
          <dd>{workflow.selected_file.browser_file?.name ?? 'None'}</dd>
        </div>
        <div>
          <dt>Size</dt>
          <dd>{fileSizeLabel(workflow.selected_file.browser_file?.size ?? 0)}</dd>
        </div>
      </dl>
    </article>
  </section>

  <section class="summary-grid" aria-label="Observation summary">
    <article>
      <h2>Counters</h2>
      <dl class="metric-grid">
        <div>
          <dt>Parsed</dt>
          <dd>{workflow.loaded_file.counters.parsed_candidates}</dd>
        </div>
        <div>
          <dt>Total</dt>
          <dd>{workflow.loaded_file.counters.total_entries}</dd>
        </div>
        <div>
          <dt>Visible</dt>
          <dd>{workflow.loaded_file.counters.visible_entries}</dd>
        </div>
        <div>
          <dt>Ignored</dt>
          <dd>{workflow.loaded_file.counters.ignored_lines}</dd>
        </div>
        <div>
          <dt>Malformed</dt>
          <dd>{workflow.loaded_file.counters.malformed_lines}</dd>
        </div>
      </dl>
    </article>

    <article>
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
    </article>

    <article>
      <h2>Observed Events</h2>
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
    </article>
  </section>

  <section class="display-grid" aria-label="Parsed display scaffold">
    <article>
      <div class="panel-heading">
        <h2>Entries</h2>
        <span class="count">{workflow.observations.entries.length}</span>
      </div>

      {#if workflow.observations.entries.length === 0}
        <p class="empty-text">No parsed entries to display.</p>
      {:else}
        <ol class="scaffold-list">
          {#each workflow.observations.entries as entry, index}
            <li>
              <div class="item-heading">
                <span>{index + 1}</span>
                <strong>{entry.label}</strong>
                <code>{entry.kind}</code>
              </div>
              <p>{previewContent(entry.content)}</p>
            </li>
          {/each}
        </ol>
      {/if}
    </article>

    <article>
      <div class="panel-heading">
        <h2>Transcript Blocks</h2>
        <span class="count">{workflow.observations.transcript_blocks.length}</span>
      </div>

      {#if workflow.observations.transcript_blocks.length === 0}
        <p class="empty-text">No transcript blocks to display.</p>
      {:else}
        <ol class="scaffold-list">
          {#each workflow.observations.transcript_blocks as block, index}
            <li>
              <div class="item-heading">
                <span>{index + 1}</span>
                <strong>{block.title}</strong>
                <code>{block.entry_type}</code>
              </div>
              <p>{previewContent(block.content)}</p>
            </li>
          {/each}
        </ol>
      {/if}
    </article>
  </section>
</main>
