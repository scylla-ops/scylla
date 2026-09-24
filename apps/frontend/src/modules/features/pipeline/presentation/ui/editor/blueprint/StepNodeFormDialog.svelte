<script lang="ts">
  import { ScyllaDialog } from '@shared/presentation/ui';
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

<ScyllaDialog
  {open}
  {onOpenChange}
  class="max-w-5xl"
  title={isEditMode ? t(pipelineMessages.editNodeTitle) : t(pipelineMessages.addNodeTitle)}
  description={isEditMode
    ? t(pipelineMessages.editNodeDescription)
    : t(pipelineMessages.addNodeDescription)}
>
  <StepNodeForm
    {editingStep}
    {projectId}
    onSubmit={handleSubmit}
    onCancel={() => onOpenChange(false)}
  />
</ScyllaDialog>
