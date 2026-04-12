import { Button } from '@/modules/shared/presentation/ui/shadcn';
import { TabsList, TabsTrigger } from '@/modules/shared/presentation/ui/shadcn/tabs.tsx';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import { useScriptStore } from '@/modules/features/pipeline-creation/presentation/stores/useScript.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { Trans, useLingui } from '@lingui/react/macro';

export const PipelineCreationTopbar = () => {
  const { t } = useLingui();
  const script = useScriptStore(state => state.script);
  const createPipeline = useDependencies().pipelineCreation.createPipelineUseCase;
  return (
    <div className={'flex justify-between w-full'}>
      <TabsList>
        <TabsTrigger value='scripting'><Trans>Scripting</Trans></TabsTrigger>
        <TabsTrigger value='blueprint'><Trans>Blueprint</Trans></TabsTrigger>
      </TabsList>
      <Button
        onClick={() => {
          createPipeline.execute(script).then(res => {
            res.fold({
              onSuccess: () => toast.success(t`Pipeline created successfully`),
              onError: err => toast.error(err.message),
            });
          });
        }}
      >
        <Trans>Create</Trans>
      </Button>
    </div>
  );
};
