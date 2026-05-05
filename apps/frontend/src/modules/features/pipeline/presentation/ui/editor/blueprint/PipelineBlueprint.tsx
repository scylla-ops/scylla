import { useCallback, useRef } from 'react';
import { BlueprintToolbar } from './BlueprintToolbar.tsx';
import { BlueprintCanvas, type BlueprintCanvasHandle } from './BlueprintCanvas.tsx';
import { StepNodeFormDialog } from './StepNodeFormDialog.tsx';
import { StartNodeFormDialog } from './StartNodeFormDialog.tsx';
import { type PipelineNodeData } from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';
import type { PipelineStep } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import { useDialog } from '@shared/presentation/hooks/use-dialog.ts';

interface PipelineBlueprintProps {
  steps: PipelineStep[];
  pipelineName: string;
  onStepsChange: (steps: PipelineStep[]) => void;
  onNameChange: (name: string) => void;
}

export const PipelineBlueprint = ({ steps, pipelineName, onStepsChange, onNameChange }: PipelineBlueprintProps) => {
  const canvasRef = useRef<BlueprintCanvasHandle>(null);

  const addStepDialog = useDialog();
  const editStepDialog = useDialog<PipelineNodeData>();
  const editStartDialog = useDialog();

  const handleAddNode = useCallback(
    (nodeId: string, command: string, args: string[]) => canvasRef.current?.addNode(nodeId, command, args),
    [],
  );

  const handleEditNode = useCallback(
    (originalId: string, nodeId: string, command: string, args: string[]) => canvasRef.current?.editNode(originalId, nodeId, command, args),
    [],
  );

  return (
    <div className='h-full w-full relative'>
      <BlueprintToolbar onAddNode={addStepDialog.open} />

      <BlueprintCanvas
        ref={canvasRef}
        steps={steps}
        pipelineName={pipelineName}
        onStepsChange={onStepsChange}
        onStartNodeDoubleClick={editStartDialog.open}
        onStepNodeDoubleClick={editStepDialog.open}
      />

      <StepNodeFormDialog
        open={addStepDialog.isOpen}
        onOpenChange={() => addStepDialog.close()}
        onAdd={handleAddNode}
        onEdit={handleEditNode}
      />

      <StepNodeFormDialog
        open={editStepDialog.isOpen}
        onOpenChange={() => editStepDialog.close()}
        editingNode={editStepDialog.data}
        onAdd={handleAddNode}
        onEdit={handleEditNode}
      />

      <StartNodeFormDialog
        open={editStartDialog.isOpen}
        onOpenChange={() => editStartDialog.close()}
        currentName={pipelineName}
        onSave={onNameChange}
      />
    </div>
  );
};
