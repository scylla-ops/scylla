import type { ReactNode } from 'react';
import { Badge, Button } from '@shadcn';
import { TabsList, TabsTrigger } from '@shadcn/tabs.tsx';
import { Trans } from '@lingui/react/macro';
import { Loader2 } from 'lucide-react';

interface PipelineCreationTopbarProps {
  onSubmit: () => void;
  submitLabel: ReactNode;
  isDirty: boolean;
  isSaving?: boolean;
}

export const PipelineEditorHeader = ({
  onSubmit,
  submitLabel,
  isDirty,
  isSaving = false,
}: PipelineCreationTopbarProps) => {
  return (
    <div className='flex w-full items-center justify-between gap-3'>
      <TabsList>
        <TabsTrigger value='scripting'>
          <Trans>Scripting</Trans>
        </TabsTrigger>
        <TabsTrigger value='blueprint'>
          <Trans>Blueprint</Trans>
        </TabsTrigger>
      </TabsList>

      <div className='flex items-center gap-2'>
        <Badge variant={isDirty ? 'outline' : 'secondary'} className='whitespace-nowrap'>
          {isDirty ? <Trans>Unsaved changes</Trans> : <Trans>Saved</Trans>}
        </Badge>
        <span className='hidden text-sm text-muted-foreground md:inline-flex'>
          {isDirty ? (
            <Trans>Save to publish this pipeline</Trans>
          ) : (
            <Trans>Ready to run after saving</Trans>
          )}
        </span>
        <Button onClick={onSubmit} disabled={isSaving}>
          {isSaving ? <Loader2 className='mr-2 size-4 animate-spin' /> : null}
          {submitLabel}
        </Button>
      </div>
    </div>
  );
};
