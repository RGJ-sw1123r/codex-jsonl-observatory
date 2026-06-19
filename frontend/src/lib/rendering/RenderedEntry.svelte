<script lang="ts">
  import type { RenderedEntryDto } from '../parse-contract'
  import { renderLabelForKind } from './render-labels'

  interface Props {
    entry: RenderedEntryDto
    index: number
  }

  let { entry, index }: Props = $props()

  const label = $derived(renderLabelForKind(entry.kind, entry.label))
</script>

<li class="render-block" data-family={label.family}>
  <header class="render-header">
    <span class="render-index">{index + 1}</span>
    <strong>{label.label}</strong>
    <span>{label.source_label}</span>
    <code>{entry.kind}</code>
  </header>
  <pre>{entry.content || '(empty)'}</pre>
</li>
