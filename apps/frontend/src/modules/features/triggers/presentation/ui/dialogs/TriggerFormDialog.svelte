<script lang="ts">
  import { Dialog, DialogContent } from '@shadcn';
  import type { CreatedTrigger, TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
  import TriggerForm from './TriggerForm.svelte';

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

<!--
  Create/edit a trigger. Kind is chosen on create and immutable on edit.

  `{#key open}` is what resets the form: the fields live in `TriggerForm`, so
  recreating it re-seeds every one of them from `trigger` with no effect
  watching `[open, trigger]` — the same rule `CLAUDE.md` gives React, spelled the
  same way here.
-->
<Dialog {open} {onOpenChange}>
  {#key open}
    <DialogContent class="sm:max-w-lg">
      <TriggerForm {pipelineId} {trigger} {onCreated} onDone={() => onOpenChange(false)} />
    </DialogContent>
  {/key}
</Dialog>
