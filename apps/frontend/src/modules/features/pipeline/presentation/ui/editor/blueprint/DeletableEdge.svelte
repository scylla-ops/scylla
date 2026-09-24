<script lang="ts">
  import Trash2Icon from '@lucide/svelte/icons/trash-2';
  import {
    BaseEdge,
    EdgeLabel,
    getSmoothStepPath,
    useSvelteFlow,
    type EdgeProps,
  } from '@xyflow/svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { EDGE_COLOR } from '../../../utils/blueprint-converter.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  const SELECTED_COLOR = 'oklch(70% 0.3 30)';

  let {
    id,
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
    selected,
    markerEnd,
  }: EdgeProps = $props();

  const { deleteElements } = useSvelteFlow();

  const path = $derived(
    getSmoothStepPath({ sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition }),
  );
  const color = $derived(selected ? SELECTED_COLOR : EDGE_COLOR);
</script>

<!--
  A dependency between two steps, with a delete button that appears once it is
  selected. `style` is a CSS string here where reactflow took an object — the
  one difference the port made to this component.
-->
<BaseEdge
  {id}
  path={path[0]}
  {markerEnd}
  style="stroke: {color}; stroke-width: {selected ? 3 : 2};{selected
    ? ` filter: drop-shadow(0 0 4px ${SELECTED_COLOR});`
    : ''}"
/>

{#if selected}
  <EdgeLabel x={path[1]} y={path[2]} transparent class="nodrag nopan" style="pointer-events: all; z-index: 1000;">
    <button
      type="button"
      aria-label={t(pipelineMessages.deleteEdge)}
      class="flex h-full w-full cursor-pointer items-center justify-center rounded-full border-2 border-background bg-destructive p-1 text-white shadow-lg transition-transform hover:scale-125"
      onclick={event => {
        event.stopPropagation();
        event.preventDefault();
        void deleteElements({ edges: [{ id }] });
      }}
      onmousedown={event => event.stopPropagation()}
    >
      <Trash2Icon class="h-6.5 w-6.5" />
    </button>
  </EdgeLabel>
{/if}
