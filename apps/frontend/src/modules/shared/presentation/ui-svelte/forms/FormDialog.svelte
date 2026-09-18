<script lang="ts" generics="TId extends string">
  import {
    Button,
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import ScyllaForm from './ScyllaForm.svelte';
  import { formDialogMessages } from './form-dialog.messages.ts';
  import type { FormItem, FormValues } from './scylla-form.struct.ts';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    title: string;
    description?: string;
    items: readonly FormItem<TId>[];
    isPending?: boolean;
    submitLabel?: string;
    pendingLabel?: string;
    onSubmit: (values: FormValues<TId>) => void;
    hideCancel?: boolean;
  }

  let {
    open,
    onOpenChange,
    title,
    description,
    items,
    isPending = false,
    submitLabel,
    pendingLabel,
    onSubmit,
    hideCancel = false,
  }: Props = $props();
</script>

<Dialog {open} {onOpenChange}>
  <!-- `hideCancel` also hides the content's own close button, so the dialog has
       exactly one way out: submitting it. -->
  <DialogContent class={hideCancel ? '[&>button]:hidden' : ''}>
    <DialogHeader>
      <DialogTitle>{title}</DialogTitle>
      {#if description}
        <DialogDescription>{description}</DialogDescription>
      {/if}
    </DialogHeader>
    <ScyllaForm {items} {isPending} {onSubmit}>
      {#snippet footer({ isValid, isPending: pending })}
        <DialogFooter>
          {#if !hideCancel}
            <Button
              type="button"
              variant="outline"
              onclick={() => onOpenChange(false)}
              disabled={pending}
            >
              {t(formDialogMessages.cancel)}
            </Button>
          {/if}
          <Button type="submit" disabled={!isValid || pending}>
            {pending
              ? (pendingLabel ?? t(formDialogMessages.creating))
              : (submitLabel ?? t(formDialogMessages.create))}
          </Button>
        </DialogFooter>
      {/snippet}
    </ScyllaForm>
  </DialogContent>
</Dialog>
