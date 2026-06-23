import { useEffect, useState } from 'react';
import {
  Badge,
  Button,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@shadcn/dialog.tsx';
import { Trans } from '@lingui/react/macro';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type {
  CreatedTrigger,
  TriggerEntity,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { useCreateTrigger } from '@/modules/features/triggers/presentation/hooks/use-create-trigger.ts';
import { useUpdateTrigger } from '@/modules/features/triggers/presentation/hooks/use-update-trigger.ts';
import { TriggerInputsEditor } from '@/modules/features/triggers/presentation/ui/dialogs/TriggerInputsEditor.tsx';
import { CronScheduleBuilder } from '@/modules/features/triggers/presentation/ui/dialogs/CronScheduleBuilder.tsx';
import {
  buildTriggerDraft,
  isCronExpressionValid,
  triggerToDraftInputs,
  type DraftInput,
  convertCronToLocal,
} from '@/modules/features/triggers/presentation/utils/trigger-form.utils.ts';

interface TriggerFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  pipelineId: string;
  /** Present => edit mode (kind is locked). Absent => create mode. */
  trigger?: TriggerEntity;
  /** Called after a successful create, so the caller can reveal a webhook secret. */
  onCreated?: (created: CreatedTrigger) => void;
}

/** Create/edit a trigger. Kind is chosen on create and immutable on edit. */
export const TriggerFormDialog = ({
  open,
  onOpenChange,
  pipelineId,
  trigger,
  onCreated,
}: TriggerFormDialogProps) => {
  const isEdit = !!trigger;
  const createTrigger = useCreateTrigger(pipelineId);
  const updateTrigger = useUpdateTrigger(pipelineId);
  const isPending = createTrigger.isPending || updateTrigger.isPending;

  const [name, setName] = useState('');
  const [kind, setKind] = useState<TriggerKind>(TriggerKind.Cron);
  const [cronExpression, setCronExpression] = useState('');
  const [signatureHeader, setSignatureHeader] = useState('');
  const [inputs, setInputs] = useState<DraftInput[]>([]);

  // Reset the form whenever the dialog (re)opens, seeded from the edited trigger.
  useEffect(() => {
    if (!open) return;
    setName(trigger?.name ?? '');
    setKind(trigger?.source.kind ?? TriggerKind.Cron);

    setCronExpression(
      trigger?.source.kind === TriggerKind.Cron ? trigger.source.expression : '0 9 * * *',
    );
    setSignatureHeader(
      trigger?.source.kind === TriggerKind.Webhook ? trigger.source.signatureHeader : '',
    );
    setInputs(triggerToDraftInputs(trigger));
  }, [open, trigger]);

  const isValid =
    name.trim().length > 0 &&
    (kind === TriggerKind.Cron ? isCronExpressionValid(cronExpression) : true);

  const handleSubmit = async () => {
    const draft = buildTriggerDraft({ name, kind, cronExpression, signatureHeader, inputs });
    try {
      if (isEdit && trigger) {
        await updateTrigger.mutateAsync({ triggerId: trigger.id, draft });
      } else {
        const created = await createTrigger.mutateAsync(draft);
        onCreated?.(created);
      }
      onOpenChange(false);
    } catch {
      // Toast shown by the global MutationCache onError handler.
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='sm:max-w-lg'>
        <DialogHeader>
          <DialogTitle>
            {isEdit ? <Trans>Edit trigger</Trans> : <Trans>New trigger</Trans>}
          </DialogTitle>
          <DialogDescription>
            <Trans>A trigger starts a run of this pipeline automatically.</Trans>
          </DialogDescription>
        </DialogHeader>

        <div className='space-y-4'>
          <div className='space-y-1'>
            <Label htmlFor='trigger-name'>
              <Trans>Name</Trans>
            </Label>
            <Input
              id='trigger-name'
              value={name}
              placeholder='nightly-build'
              onChange={e => setName(e.target.value)}
            />
          </div>

          {isEdit ? (
            <div className='space-y-1'>
              <Label>
                <Trans>Type</Trans>
              </Label>
              <div className='flex items-center gap-2'>
                <Badge variant='secondary'>{kind === TriggerKind.Cron ? 'Cron' : 'Webhook'}</Badge>
                <span className='text-xs text-muted-foreground'>
                  <Trans>Type can't be changed — delete and recreate to switch.</Trans>
                </span>
              </div>
            </div>
          ) : (
            <div className='space-y-1'>
              <Label htmlFor='trigger-kind'>
                <Trans>Type</Trans>
              </Label>
              <Select value={kind} onValueChange={value => setKind(value as TriggerKind)}>
                <SelectTrigger id='trigger-kind' className='w-full'>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={TriggerKind.Cron}>Cron</SelectItem>
                  <SelectItem value={TriggerKind.Webhook}>Webhook</SelectItem>
                </SelectContent>
              </Select>
            </div>
          )}

          {kind === TriggerKind.Cron ? (
            <div className='space-y-1'>
              <Label>
                <Trans>Schedule</Trans>
              </Label>
              <CronScheduleBuilder
                key={trigger?.id ?? 'new'}
                initialValue={
                  trigger?.source.kind === TriggerKind.Cron
                    ? convertCronToLocal(trigger.source.expression)
                    : '0 9 * * *'
                }
                onChange={setCronExpression}
              />
            </div>
          ) : (
            <div className='space-y-1'>
              <Label htmlFor='trigger-header'>
                <Trans>Signature header</Trans>
              </Label>
              <Input
                id='trigger-header'
                value={signatureHeader}
                placeholder='X-Hub-Signature-256'
                onChange={e => setSignatureHeader(e.target.value)}
              />
              <p className='text-xs text-muted-foreground'>
                <Trans>
                  Header carrying the HMAC signature. Leave empty for the Scylla default.
                </Trans>
              </p>
            </div>
          )}

          <TriggerInputsEditor
            inputs={inputs}
            onChange={setInputs}
            allowJsonPointer={kind === TriggerKind.Webhook}
          />
        </div>

        <DialogFooter>
          <Button
            type='button'
            variant='outline'
            onClick={() => onOpenChange(false)}
            disabled={isPending}
          >
            <Trans>Cancel</Trans>
          </Button>
          <Button type='button' onClick={handleSubmit} disabled={!isValid || isPending}>
            {isEdit ? <Trans>Save</Trans> : <Trans>Create</Trans>}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
