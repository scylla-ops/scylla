<script lang="ts">
  import PlayIcon from '@lucide/svelte/icons/play';
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';
  import { Card } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { BlueprintStartNode } from '../../../utils/blueprint-converter.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  let { data }: NodeProps<BlueprintStartNode> = $props();
</script>

<!-- The pipeline itself, as the graph's single entry point: every step with no
     dependency hangs off it. Not deletable — see `stepsToFlow`. -->
<Card
  class="w-55 cursor-pointer overflow-hidden border-primary/30 p-0 shadow-md transition-all duration-200 hover:scale-105 hover:border-primary hover:shadow-lg"
>
  <div class="flex items-center gap-2 bg-primary px-3 py-2">
    <PlayIcon class="h-4 w-4 fill-primary-foreground text-primary-foreground" />
    <span class="text-sm font-bold text-primary-foreground">{t(pipelineMessages.pipeline)}</span>
  </div>

  <div class="px-3 py-3">
    <span class="block truncate text-sm font-medium">
      {data.name || t(pipelineMessages.unnamedPipeline)}
    </span>
  </div>

  <Handle
    type="source"
    position={Position.Right}
    class="h-3! w-3! border-2! border-background! bg-primary!"
  />
</Card>
