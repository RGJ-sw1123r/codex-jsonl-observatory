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
</script>

<main class="control-room">
  <section class="load-panel" aria-labelledby="load-title">
    <div>
      <p class="eyebrow">Codex JSONL Observatory</p>
      <h1 id="load-title">Load Workflow Scaffold</h1>
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
      <button
        type="button"
        disabled={workflow.status === 'idle'}
        onclick={markLoadBoundary}
      >
        Mark load boundary
      </button>
      <span class="status" data-status={workflow.status}>{workflow.status}</span>
    </div>
  </section>

  <section class="state-grid" aria-label="Load workflow state">
    <article>
      <h2>Selected File</h2>
      <dl>
        <div>
          <dt>Path</dt>
          <dd>{workflow.selected_file.path || 'None'}</dd>
        </div>
        <div>
          <dt>Name</dt>
          <dd>{workflow.selected_file.browser_file?.name ?? 'None'}</dd>
        </div>
        <div>
          <dt>Size</dt>
          <dd>{workflow.selected_file.browser_file?.size ?? 0}</dd>
        </div>
      </dl>
    </article>

    <article>
      <h2>Loaded Metadata</h2>
      <dl>
        <div>
          <dt>File</dt>
          <dd>{workflow.loaded_file.metadata?.file_name ?? 'Not loaded'}</dd>
        </div>
        <div>
          <dt>Session</dt>
          <dd>{workflow.loaded_file.metadata?.session_id ?? 'None'}</dd>
        </div>
        <div>
          <dt>Resume</dt>
          <dd>{workflow.loaded_file.metadata?.resume_command ?? 'None'}</dd>
        </div>
      </dl>
    </article>

    <article>
      <h2>Counters</h2>
      <dl class="compact">
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
      <h2>Observations</h2>
      <dl class="compact">
        <div>
          <dt>Entries</dt>
          <dd>{workflow.observations.entries.length}</dd>
        </div>
        <div>
          <dt>Transcript blocks</dt>
          <dd>{workflow.observations.transcript_blocks.length}</dd>
        </div>
        <div>
          <dt>Event counts</dt>
          <dd>{workflow.loaded_file.observed_event_counts.length}</dd>
        </div>
      </dl>
    </article>

    <article>
      <h2>Error</h2>
      <dl>
        <div>
          <dt>Code</dt>
          <dd>{workflow.error?.code ?? 'None'}</dd>
        </div>
        <div>
          <dt>Message</dt>
          <dd>{workflow.error?.message ?? 'None'}</dd>
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
  </section>
</main>
