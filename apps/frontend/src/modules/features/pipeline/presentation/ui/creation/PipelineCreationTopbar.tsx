import { Button } from '@shadcn';
import { TabsList, TabsTrigger } from '@shadcn/tabs.tsx';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';
import { Trans } from '@lingui/react/macro';
import { useCallback } from 'react';
import { useCreatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-create-pipeline.ts';
import { useEditPipeline } from '@/modules/features/pipeline/presentation/hooks/use-edit-pipeline.ts';
import type { Pipeline } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

interface PipelineCreationTopbarProps {
  isEditing: boolean;
  pipelineId?: string;
}

export const PipelineCreationTopbar = ({ isEditing, pipelineId }: PipelineCreationTopbarProps) => {
  const script = useScriptStore(state => state.script);
  const createPipeline = useCreatePipeline();
  const editPipeline = useEditPipeline();

  const onCreatePipeline = useCallback(() => {
    createPipeline.mutate(script);
  }, [createPipeline, script]);

  const onEditPipeline = useCallback(() => {
    if (!script || !pipelineId) return;

    try {
      const serializedScript: Pipeline = JSON.parse(script);
      const nodes = serializedScript.nodes;
      const pipelineName = serializedScript.name;

      editPipeline.mutate({ id: pipelineId, nodes: nodes, name: pipelineName });
    } catch (error) {
      console.error('Error parsing script:', error);
    }
  }, [editPipeline, pipelineId, script]);

  return (
    <div className={'flex justify-between w-full'}>
      <TabsList>
        <TabsTrigger value='scripting'>
          <Trans>Scripting</Trans>
        </TabsTrigger>
        <TabsTrigger value='blueprint'>
          <Trans>Blueprint</Trans>
        </TabsTrigger>
      </TabsList>
      <Button
        onClick={() => {
          if (isEditing) {
            onEditPipeline();
          } else {
            onCreatePipeline();
          }
        }}
      >
        <Trans>{isEditing ? 'Edit' : 'Create'}</Trans>
      </Button>
    </div>
  );
};
