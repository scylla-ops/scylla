import { Button } from '@shadcn';
import { TabsList, TabsTrigger } from '@shadcn/tabs.tsx';

export const PipelineCreationTopbar = () => {
  return (
    <div className={'flex justify-between w-full'}>
      <TabsList>
        <TabsTrigger value='scripting'>Scripting</TabsTrigger>
        <TabsTrigger value='blueprint'>Blueprint</TabsTrigger>
      </TabsList>
      <Button>Create</Button>
    </div>
  );
};
