<script lang="ts">
  import type { RenderedEntryDto } from '../parse-contract'
  import { renderLabelForKind } from './render-labels'

  interface Props {
    entry: RenderedEntryDto
    index: number
  }

  let { entry, index }: Props = $props()
  let isExpanded = $state(false)

  const label = $derived(renderLabelForKind(entry.kind, entry.label))
</script>

<li class="render-block" data-family={label.family}>
  <button
    type="button"
    class="render-header raw-entry-toggle"
    aria-expanded={isExpanded}
    onclick={() => (isExpanded = !isExpanded)}
  >
    <span class="render-index">{index + 1}</span>
    <strong>{label.label}</strong>
    <span>{label.source_label}</span>
    <code>{entry.kind}</code>
  </button>
  {#if isExpanded}
    <pre>{entry.content || '(empty)'}</pre>
  {/if}
</li>
