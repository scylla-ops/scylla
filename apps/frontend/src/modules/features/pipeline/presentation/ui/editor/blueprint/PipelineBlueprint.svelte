<script lang="ts">
  import PlusIcon from '@lucide/svelte/icons/plus';
  import { Button } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { PipelineStep } from '../../../../domain/structs/pipeline.struct.ts';
  import { createBlueprintState } from '../../../blueprint.state.svelte.ts';
  import BlueprintCanvas from './BlueprintCanvas.svelte';
  import StartNodeFormDialog from './StartNodeFormDialog.svelte';
  import StepNodeFormDialog from './StepNodeFormDialog.svelte';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  interface Props {
    steps: PipelineStep[];
    pipelineName: string;
    projectId?: string;
    onStepsChange: (steps: PipelineStep[]) => void;
    onNameChange: (name: string) => void;
  }

  let { steps, pipelineName, projectId, onStepsChange, onNameChange }: Props = $props();

  const blueprint = createBlueprintState({
    steps: () => steps,
    pipelineName: () => pipelineName,
    // Read at call time, so a swapped callback is the one called.
    onStepsChange: next => onStepsChange(next),
  });

  let editingStep = $state<PipelineStep | undefined>(undefined);
  let stepDialogOpen = $state(false);
  let nameDialogOpen = $state(false);
</script>

<div class="relative h-full w-full">
  <div class="absolute top-3 right-3 z-10 flex gap-2">
    <Button
      size="sm"
      variant="outline"
      onclick={() => {
        editingStep = undefined;
        stepDialogOpen = true;
      }}
    >
      <PlusIcon class="mr-1 h-4 w-4" />
      {t(pipelineMessages.addNode)}
    </Button>
  </div>

  <BlueprintCanvas
    {blueprint}
    onStartNodeDoubleClick={() => (nameDialogOpen = true)}
    onStepNodeDoubleClick={step => {
      editingStep = step;
      stepDialogOpen = true;
    }}
  />

  <!-- One dialog for adding and editing: `editingStep` tells them apart. -->
  <StepNodeFormDialog
    open={stepDialogOpen}
    onOpenChange={open => (stepDialogOpen = open)}
    {editingStep}
    {projectId}
    onAdd={(nodeId, value) => blueprint.addNode(nodeId, value)}
    onEdit={(originalId, nodeId, value) => blueprint.editNode(originalId, nodeId, value)}
  />

  <StartNodeFormDialog
    open={nameDialogOpen}
    onOpenChange={open => (nameDialogOpen = open)}
    currentName={pipelineName}
    onSave={onNameChange}
  />
</div>
