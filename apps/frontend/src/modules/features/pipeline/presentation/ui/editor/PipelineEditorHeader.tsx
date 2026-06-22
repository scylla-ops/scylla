import { Button } from '@shadcn';
import { TabsList, TabsTrigger } from '@shadcn/tabs.tsx';
import { Trans } from '@lingui/react/macro';

interface PipelineCreationTopbarProps {
  onSubmit: () => void;
  submitLabel: string;
  /** Disables the submit button (e.g. when the script JSON is invalid). */
  submitDisabled?: boolean;
  /** Disables the blueprint tab so an invalid script can't be silently overwritten. */
  blueprintDisabled?: boolean;
}

export const PipelineEditorHeader = ({
  onSubmit,
  submitLabel,
  submitDisabled,
  blueprintDisabled,
}: PipelineCreationTopbarProps) => {
  return (
    <div className={'flex justify-between w-full'}>
      <TabsList>
        <TabsTrigger value='scripting'>
          <Trans>Scripting</Trans>
        </TabsTrigger>
        <TabsTrigger value='blueprint' disabled={blueprintDisabled}>
          <Trans>Blueprint</Trans>
        </TabsTrigger>
      </TabsList>
      <Button onClick={onSubmit} disabled={submitDisabled}>
        <Trans>{submitLabel}</Trans>
      </Button>
    </div>
  );
};
