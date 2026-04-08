import { Button } from '@/modules/shared/presentation/ui/shadcn';
import { TabsList, TabsTrigger } from '@/modules/shared/presentation/ui/shadcn/tabs.tsx';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import { useScriptStore } from '@/modules/features/pipeline-creation/presentation/stores/useScript.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const PipelineCreationTopbar = () => {
  const script = useScriptStore(state => state.script);
  const createPipeline = useDependencies().pipelineCreation.createPipelineUseCase;
  return (
    <div className={'flex justify-between w-full'}>
      <TabsList>
        <TabsTrigger value='scripting'>Scripting</TabsTrigger>
        <TabsTrigger value='blueprint'>Blueprint</TabsTrigger>
      </TabsList>
      <Button
        onClick={() => {
          createPipeline.execute(script).then(res => {
            res.fold({
              onSuccess: () => toast.success('Pipeline created successfully'),
              onError: err => toast.error(err.message),
            });
          });
        }}
      >
        Create
      </Button>
    </div>
  );
};
