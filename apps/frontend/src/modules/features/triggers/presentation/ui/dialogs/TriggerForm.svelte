<script lang="ts">
  import {
    Badge,
    Button,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    Input,
    Label,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@shadcn';
  import { createMutation } from '@platform/query';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { CreatedTrigger, TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
  import {
    TriggerKind,
    type TriggerDraftKind,
  } from '../../../domain/structs/trigger-source.struct.ts';
  import { triggerMutations } from '../../triggers.queries.ts';
  import {
    buildTriggerDraft,
    convertCronToLocal,
    isCronExpressionValid,
    triggerToDraftInputs,
    type DraftInput,
  } from '../../utils/trigger-form.utils.ts';
  import CronScheduleBuilder from './CronScheduleBuilder/CronScheduleBuilder.svelte';
  import TriggerInputsEditor from './TriggerInputsEditor/TriggerInputsEditor.svelte';
  import { triggersMessages } from '../triggers.messages.ts';

  interface Props {
    pipelineId: string;
    /** Present => edit mode (kind is locked). Absent => create mode. */
    trigger?: TriggerEntity;
    onCreated?: (created: CreatedTrigger) => void;
    onDone: () => void;
  }

  let { pipelineId, trigger, onCreated, onDone }: Props = $props();

  const createTrigger = createMutation(() => triggerMutations.create(pipelineId));
  const updateTrigger = createMutation(() => triggerMutations.update(pipelineId));

  // svelte-ignore state_referenced_locally
  const isEdit = !!trigger;
  const isPending = $derived(createTrigger.isPending || updateTrigger.isPending);

  /**
   * Seeded from the edited trigger at creation time, and that is the whole
   * reason this is a component of its own: the dialog recreates it on every
   * open, so these initializers *are* the reset the React version needed an
   * effect on `[open, trigger]` for.
   */
  const DEFAULT_CRON = '0 9 * * *';

   
  // svelte-ignore state_referenced_locally
  let name = $state(trigger?.name ?? '');
  // svelte-ignore state_referenced_locally
  let kind = $state<TriggerDraftKind>(
    trigger?.source.kind === TriggerKind.Webhook ? TriggerKind.Webhook : TriggerKind.Cron,
  );
  // svelte-ignore state_referenced_locally
  let cronExpression = $state(
    trigger?.source.kind === TriggerKind.Cron ? trigger.source.expression : DEFAULT_CRON,
  );
  // svelte-ignore state_referenced_locally
  let signatureHeader = $state(
    trigger?.source.kind === TriggerKind.Webhook ? trigger.source.signatureHeader : '',
  );
  // svelte-ignore state_referenced_locally
  let inputs = $state<DraftInput[]>(triggerToDraftInputs(trigger));
   

  /** A source arm newer than this build can't be rendered, so it can't be re-sent either. */
  // svelte-ignore state_referenced_locally
  const isUnknownSource = trigger?.source.kind === TriggerKind.Unknown;

  const isValid = $derived(
    !isUnknownSource &&
      name.trim().length > 0 &&
      (kind === TriggerKind.Cron ? isCronExpressionValid(cronExpression) : true),
  );

  const initialCron = $derived(
    trigger?.source.kind === TriggerKind.Cron
      ? convertCronToLocal(trigger.source.expression)
      : DEFAULT_CRON,
  );

  const handleSubmit = async () => {
    const draft = buildTriggerDraft({ name, kind, cronExpression, signatureHeader, inputs });

    try {
      if (isEdit && trigger) {
        await updateTrigger.mutateAsync({ triggerId: trigger.id, draft });
      } else {
        // Awaited into a binding first: `onCreated?.(await …)` would skip the
        // argument entirely when no callback is passed — creating nothing while
        // still closing the dialog as though it had worked.
        const created = await createTrigger.mutateAsync(draft);
        onCreated?.(created);
      }
      onDone();
    } catch {
      // Toast shown by the global MutationCache onError handler.
    }
  };
</script>

<DialogHeader>
  <DialogTitle>
    {isEdit ? t(triggersMessages.editTrigger) : t(triggersMessages.newTrigger)}
  </DialogTitle>
  <DialogDescription>{t(triggersMessages.formDescription)}</DialogDescription>
</DialogHeader>

<div class="space-y-4">
  <div class="space-y-1">
    <Label for="trigger-name">{t(triggersMessages.name)}</Label>
    <Input
      id="trigger-name"
      value={name}
      placeholder="nightly-build"
      oninput={event => (name = event.currentTarget.value)}
    />
  </div>

  {#if isEdit}
    <div class="space-y-1">
      <Label>{t(triggersMessages.type)}</Label>
      <div class="flex items-center gap-2">
        <Badge variant="secondary">
          {isUnknownSource
            ? t(triggersMessages.unknown)
            : kind === TriggerKind.Cron
              ? 'Cron'
              : 'Webhook'}
        </Badge>
        <span class="text-xs text-muted-foreground">{t(triggersMessages.typeLocked)}</span>
      </div>
    </div>
  {:else}
    <div class="space-y-1">
      <Label for="trigger-kind">{t(triggersMessages.type)}</Label>
      <Select
        type="single"
        value={kind}
        onValueChange={value => (kind = value as TriggerDraftKind)}
      >
        <SelectTrigger id="trigger-kind" class="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={TriggerKind.Cron} label="Cron">Cron</SelectItem>
          <SelectItem value={TriggerKind.Webhook} label="Webhook">Webhook</SelectItem>
        </SelectContent>
      </Select>
    </div>
  {/if}

  {#if kind === TriggerKind.Cron}
    <div class="space-y-1">
      <Label>{t(triggersMessages.schedule)}</Label>
      <CronScheduleBuilder
        initialValue={initialCron}
        onChange={expression => (cronExpression = expression)}
      />
    </div>
  {:else}
    <div class="space-y-1">
      <Label for="trigger-header">{t(triggersMessages.signatureHeader)}</Label>
      <Input
        id="trigger-header"
        value={signatureHeader}
        placeholder="X-Hub-Signature-256"
        oninput={event => (signatureHeader = event.currentTarget.value)}
      />
      <p class="text-xs text-muted-foreground">{t(triggersMessages.signatureHeaderHint)}</p>
    </div>
  {/if}

  <TriggerInputsEditor
    {inputs}
    onChange={next => (inputs = next)}
    allowJsonPointer={kind === TriggerKind.Webhook}
  />
</div>

<DialogFooter>
  <Button type="button" variant="outline" onclick={onDone} disabled={isPending}>
    {t(triggersMessages.cancel)}
  </Button>
  <Button type="button" onclick={() => void handleSubmit()} disabled={!isValid || isPending}>
    {isEdit ? t(triggersMessages.save) : t(triggersMessages.create)}
  </Button>
</DialogFooter>
