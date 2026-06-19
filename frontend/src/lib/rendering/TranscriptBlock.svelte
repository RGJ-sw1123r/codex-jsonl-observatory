<script lang="ts">
  import type { TranscriptBlockDto } from '../parse-contract'
  import { renderLabelForKind } from './render-labels'

  interface Props {
    block: TranscriptBlockDto
    index: number
  }

  let { block, index }: Props = $props()

  const label = $derived(renderLabelForKind(block.entry_type, block.label))
</script>

<li class="render-block transcript-block" data-family={label.family}>
  <header class="render-header">
    <span class="render-index">{index + 1}</span>
    <strong>{label.label}</strong>
    <span>{block.title || label.source_label}</span>
    <code>{block.entry_type}</code>
  </header>
  <pre>{block.content || '(empty)'}</pre>
</li>
