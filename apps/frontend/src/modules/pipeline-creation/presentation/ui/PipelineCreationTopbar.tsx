import { Button } from '@shadcn';
import { TabsList, TabsTrigger } from '@shadcn/tabs.tsx';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import { useScriptStore } from '@/modules/pipeline-creation/presentation/stores/useScript.ts';

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
            res.fold(
              _ => alert('success'),
              err => alert(err.message + ' ' + err.cause),
            );
          });
        }}
      >
        Create
      </Button>
    </div>
  );
};
