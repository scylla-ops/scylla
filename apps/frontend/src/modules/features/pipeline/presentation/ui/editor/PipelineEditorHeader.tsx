import { Button } from '@shadcn';
import { TabsList, TabsTrigger } from '@shadcn/tabs.tsx';
import { Trans } from '@lingui/react/macro';

interface PipelineCreationTopbarProps {
  onSubmit: () => void;
  submitLabel: string;
  isPending?: boolean;
}

export const PipelineEditorHeader = ({ onSubmit, submitLabel, isPending }: PipelineCreationTopbarProps) => {
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
      <Button onClick={onSubmit} disabled={isPending}>
        <Trans>{submitLabel}</Trans>
      </Button>
    </div>
  );
};
