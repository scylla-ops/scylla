<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { PipelineStep } from '../../../../domain/structs/pipeline.struct.ts';
  import type { NodeFormValue } from '../../../utils/blueprint-converter.ts';
  import StepNodeForm from './StepNodeForm.svelte';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    /** If provided, the dialog edits that step instead of defining a new one. */
    editingStep?: PipelineStep;
    projectId?: string;
    onAdd: (nodeId: string, value: NodeFormValue) => void;
    onEdit: (originalId: string, nodeId: string, value: NodeFormValue) => void;
  }

  let { open, onOpenChange, editingStep, projectId, onAdd, onEdit }: Props = $props();

  const isEditMode = $derived(!!editingStep);

  const handleSubmit = (nodeId: string, value: NodeFormValue) => {
    if (editingStep) onEdit(editingStep.id, nodeId, value);
    else onAdd(nodeId, value);
    onOpenChange(false);
  };
</script>

<Dialog {open} {onOpenChange}>
  <DialogContent class="max-w-5xl">
    <DialogHeader>
      <DialogTitle>
        {isEditMode ? t(pipelineMessages.editNodeTitle) : t(pipelineMessages.addNodeTitle)}
      </DialogTitle>
      <DialogDescription>
        {isEditMode
          ? t(pipelineMessages.editNodeDescription)
          : t(pipelineMessages.addNodeDescription)}
      </DialogDescription>
    </DialogHeader>

    <!-- `{#key open}` is the reset: reopening rebuilds the form, which re-runs
         its `$state` initialisers from `editingStep`. The React version used an
         effect on `[open, editingNode]` and showed the previous node's values
         for a frame. -->
    {#key open}
      <StepNodeForm
        {editingStep}
        {projectId}
        onSubmit={handleSubmit}
        onCancel={() => onOpenChange(false)}
      />
    {/key}
  </DialogContent>
</Dialog>
