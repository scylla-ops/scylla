import { Button } from '@shadcn';
import { TabsList, TabsTrigger } from '@shadcn/tabs.tsx';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { Trans, useLingui } from '@lingui/react/macro';
import { useCallback } from 'react';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';

interface PipelineCreationTopbarProps {
  isEditing: boolean;
}

export const PipelineCreationTopbar = ({ isEditing }: PipelineCreationTopbarProps) => {
  const { t } = useLingui();

  const script = useScriptStore(state => state.script);
  const createPipeline = useDependencies().pipeline.createPipeline;
  const { goToProject } = useScyllaNavigate();
  const currentProject = useContextStore(state => state.project);

  const onCreatePipeline = useCallback(() => {
    createPipeline.execute(script).then(res => {
      res.fold({
        onSuccess: () => {
          toast.success(t`Pipeline created successfully`);
          if (currentProject.name && currentProject.id) {
            goToProject({ id: currentProject.id, name: currentProject.name });
          }
        },
        onError: err => toast.error(err.message),
      });
    });
  }, [createPipeline, script, t, currentProject, goToProject]);

  const onEditPipeline = useCallback(() => {}, []);

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
