<script lang="ts">
  import {
    CopyableText,
    StatusIndicator,
    type StatusState,
  } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { formatDay } from '@shared/utils/date-utils.ts';
  import type { PipelineMetadata } from '../../../../domain/structs/pipeline.struct.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  interface Props {
    pipeline: PipelineMetadata;
    status: StatusState;
  }

  let { pipeline, status }: Props = $props();
</script>

<div class="flex w-full flex-row items-center justify-start gap-2">
  <div class="shrink-0">
    <StatusIndicator state={status} />
  </div>

  <!-- Stretched, not `items-start`, or `truncate` has nothing to cut. -->
  <div class="flex min-w-0 flex-1 flex-col text-start">
    <span class="truncate font-semibold text-foreground">{pipeline.name}</span>
    <span class="truncate text-xs text-muted-foreground">
      {t(pipelineMessages.creation)}
      {formatDay(pipeline.createdAt)}
    </span>
    <CopyableText class="text-xs text-muted-foreground/80" value={pipeline.id} />
  </div>
</div>
