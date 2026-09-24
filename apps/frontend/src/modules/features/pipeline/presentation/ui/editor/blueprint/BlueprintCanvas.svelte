<script lang="ts">
  import {
    Background,
    BackgroundVariant,
    Controls,
    SvelteFlow,
    type EdgeTypes,
    type NodeTypes,
  } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';
  import type { PipelineStep } from '../../../../domain/structs/pipeline.struct.ts';
  import { DEFAULT_EDGE_STYLE, START_NODE_ID, stepOf } from '../../../utils/blueprint-converter.ts';
  import type { BlueprintState } from '../../../blueprint.state.svelte.ts';
  import DeletableEdge from './DeletableEdge.svelte';
  import PipelineStepNode from './PipelineStepNode.svelte';
  import StartNode from './StartNode.svelte';

  interface Props {
    blueprint: BlueprintState;
    onStartNodeDoubleClick: () => void;
    onStepNodeDoubleClick: (step: PipelineStep) => void;
  }

  let { blueprint, onStartNodeDoubleClick, onStepNodeDoubleClick }: Props = $props();

  const nodeTypes: NodeTypes = { pipelineStep: PipelineStepNode, startNode: StartNode };
  const edgeTypes: EdgeTypes = { deletable: DeletableEdge };
</script>

<!-- `nodes` and `edges` are bound: the library drags, selects and deletes; the callbacks report the changes. -->
<SvelteFlow
  bind:nodes={blueprint.nodes}
  bind:edges={blueprint.edges}
  {nodeTypes}
  {edgeTypes}
  fitView
  fitViewOptions={{ padding: 0.2 }}
  deleteKey={['Backspace', 'Delete']}
  edgesFocusable
  defaultEdgeOptions={DEFAULT_EDGE_STYLE}
  class="rounded-lg"
  onbeforeconnect={connection => (blueprint.canConnect(connection) ? connection : false)}
  onconnect={() => blueprint.sync()}
  ondelete={() => blueprint.sync()}
  onnodeclick={({ node, event }) => {
    // No double-click event: the click's `detail` counts.
    if ((event as MouseEvent).detail < 2) return;
    if (node.id === START_NODE_ID) {
      onStartNodeDoubleClick();
      return;
    }
    const step = stepOf(node);
    if (step) onStepNodeDoubleClick(step);
  }}
>
  <Background variant={BackgroundVariant.Dots} gap={16} size={1} class="bg-background!" />
  <Controls class="border-border! bg-background! shadow-sm!" />
</SvelteFlow>

<style>
  :global(.svelte-flow__controls-button) {
    --xy-controls-button-background-color: var(--background);
    --xy-controls-button-background-color-hover: var(--muted);
    --xy-controls-button-color: var(--foreground);
    --xy-controls-button-color-hover: var(--foreground);
    --xy-controls-button-border-color: var(--border);
  }
</style>
