<script lang="ts">
  import type { MessageDescriptor } from '@lingui/core';
  import Loader2Icon from '@lucide/svelte/icons/loader-2';
  import { Permission, can } from '@platform/authz';
  import { TabsList, TabsTrigger } from '@shadcn';
  import { GatedButton } from '@shared/presentation/ui';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  interface Props {
    onSubmit: () => void;
    submitLabel: string;
    /** `edit` tracks the unsaved changes; `create` does not. */
    mode: 'create' | 'edit';
    submitDisabled?: boolean;
    /** So that an invalid script is not overwritten from the blueprint. */
    blueprintDisabled?: boolean;
    isDirty: boolean;
    isSaving?: boolean;
  }

  let {
    onSubmit,
    submitLabel,
    mode,
    submitDisabled = false,
    blueprintDisabled = false,
    isDirty,
    isSaving = false,
  }: Props = $props();

  type SaveStatus = 'draft' | 'saving' | 'dirty' | 'saved';

  const STATUS_LABEL: Record<SaveStatus, MessageDescriptor> = {
    draft: pipelineMessages.statusDraft,
    saving: pipelineMessages.statusSaving,
    dirty: pipelineMessages.statusDirty,
    saved: pipelineMessages.statusSaved,
  };

  const status = $derived<SaveStatus>(
    isSaving ? 'saving' : mode === 'create' ? 'draft' : isDirty ? 'dirty' : 'saved',
  );

  const canSubmit = $derived(
    can(mode === 'create' ? Permission.CREATE_PIPELINE : Permission.UPDATE_PIPELINE),
  );
</script>

<div class="flex w-full items-center justify-between gap-3">
  <TabsList>
    <TabsTrigger value="scripting">{t(pipelineMessages.scripting)}</TabsTrigger>
    <TabsTrigger value="blueprint" disabled={blueprintDisabled}>
      {t(pipelineMessages.blueprint)}
    </TabsTrigger>
  </TabsList>

  <div class="flex items-center gap-3">
    <div class="flex items-center gap-2 text-sm text-foreground/80">
      <span class="relative flex size-2">
        {#if status === 'dirty'}
          <span class="absolute inline-flex size-full animate-ping rounded-full bg-amber-500/70"
          ></span>
        {/if}
        <span
          class={cn(
            'relative inline-flex size-2 rounded-full transition-colors duration-300',
            (status === 'draft' || status === 'saving') && 'bg-muted-foreground/60',
            status === 'dirty' && 'bg-amber-500',
            status === 'saved' && 'bg-emerald-500',
          )}
        ></span>
      </span>
      <!-- `{#key}` replays the enter animation on each status change. -->
      {#key status}
        <span
          class="hidden animate-in fade-in-0 slide-in-from-bottom-1 duration-150 ease-out whitespace-nowrap md:inline-flex"
        >
          {t(STATUS_LABEL[status])}
        </span>
      {/key}
    </div>

    <GatedButton
      allowed={canSubmit}
      deniedReason={mode === 'create'
        ? t(pipelineMessages.createDenied)
        : t(pipelineMessages.editDenied)}
      onclick={onSubmit}
      disabled={submitDisabled || isSaving}
    >
      {#if isSaving}<Loader2Icon class="mr-2 size-4 animate-spin" />{/if}
      {submitLabel}
    </GatedButton>
  </div>
</div>
