<script lang="ts">
  import TerminalIcon from '@lucide/svelte/icons/terminal';
  import XIcon from '@lucide/svelte/icons/x';
  import { Handle, Position, useSvelteFlow, type NodeProps } from '@xyflow/svelte';
  import { Card } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { BlueprintStepNode } from '../../../utils/blueprint-converter.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  let { id, data, selected }: NodeProps<BlueprintStepNode> = $props();

  const { deleteElements } = useSvelteFlow();

  const step = $derived(data.step);
  const scriptPreview = $derived(
    step.kind === 'script' ? (step.script.split('\n').find(line => line.trim() !== '') ?? '') : '',
  );
</script>

<Card
  class="w-[260px] cursor-pointer overflow-hidden p-0 transition-all duration-200 hover:scale-105 hover:border-primary hover:shadow-lg {selected
    ? 'scale-105 shadow-lg ring-2 ring-primary'
    : 'shadow-sm'}"
>
  <Handle
    type="target"
    position={Position.Left}
    class="h-3! w-3! border-2! border-background! bg-primary!"
  />

  <div class="flex items-center justify-between border-b bg-primary/10 px-3 py-2">
    <div class="flex items-center gap-2">
      <TerminalIcon class="h-4 w-4 text-primary" />
      <span class="truncate text-sm font-semibold">{step.id}</span>
    </div>
    <button
      type="button"
      aria-label={t(pipelineMessages.deleteNode)}
      onclick={event => {
        event.stopPropagation();
        void deleteElements({ nodes: [{ id }] });
      }}
      class="rounded p-0.5 text-muted-foreground transition-colors hover:bg-destructive/20 hover:text-destructive"
    >
      <XIcon class="h-3.5 w-3.5" />
    </button>
  </div>

  <div class="space-y-1 px-3 py-2">
    {#if step.kind === 'exec'}
      <div class="flex items-center gap-1.5">
        <span class="text-xs text-muted-foreground">cmd:</span>
        <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{step.command}</code>
      </div>
      {#if step.args.length > 0}
        <div class="flex items-center gap-1.5">
          <span class="text-xs text-muted-foreground">args:</span>
          <code class="max-w-[180px] truncate rounded bg-muted px-1.5 py-0.5 font-mono text-xs">
            {step.args.join(' ')}
          </code>
        </div>
      {/if}
    {:else}
      <div class="flex items-center gap-1.5">
        <span
          class="rounded bg-primary/15 px-1.5 py-0.5 text-[10px] font-semibold tracking-wide text-primary uppercase"
        >
          {step.shell}
        </span>
        <code class="max-w-[180px] truncate rounded bg-muted px-1.5 py-0.5 font-mono text-xs">
          {scriptPreview}
        </code>
      </div>
    {/if}
  </div>

  <Handle
    type="source"
    position={Position.Right}
    class="h-3! w-3! border-2! border-background! bg-primary!"
  />
</Card>
