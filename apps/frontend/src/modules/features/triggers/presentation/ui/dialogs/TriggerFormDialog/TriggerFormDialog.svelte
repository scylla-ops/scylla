<script lang="ts">
  import { ScyllaDialog } from '@shared/presentation/ui';
  import type { CreatedTrigger, TriggerEntity } from '../../../../domain/entities/trigger.entity.ts';
  import TriggerForm from '../TriggerForm.svelte';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    pipelineId: string;
    /** Present => edit mode (kind is locked). Absent => create mode. */
    trigger?: TriggerEntity;
    /** Called after a successful create, so the caller can reveal a webhook secret. */
    onCreated?: (created: CreatedTrigger) => void;
  }

  let { open, onOpenChange, pipelineId, trigger, onCreated }: Props = $props();
</script>

<ScyllaDialog {open} {onOpenChange} class="sm:max-w-lg">
  <TriggerForm {pipelineId} {trigger} {onCreated} onDone={() => onOpenChange(false)} />
</ScyllaDialog>
