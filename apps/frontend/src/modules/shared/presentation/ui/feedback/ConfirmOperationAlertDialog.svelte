<script lang="ts">
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { confirmOperationMessages } from './confirm-operation.messages.ts';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onContinue: () => void;
    title?: string;
    description?: string;
    /** Disables both buttons while the operation is in flight. */
    isLoading?: boolean;
  }

  let { open, onOpenChange, onContinue, title, description, isLoading = false }: Props = $props();
</script>

<!--
  Controlled, not bound: the parent owns `open` because it also owns the mutation
  behind Continue, and the dialog has to stay up and disabled until that settles.
  That is also why the action and cancel buttons are plain buttons — see
  `shadcn/alert-dialog-action.svelte`.
-->
<AlertDialog {open} {onOpenChange}>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>{title ?? t(confirmOperationMessages.title)}</AlertDialogTitle>
      <AlertDialogDescription>
        {description ?? t(confirmOperationMessages.description)}
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel onclick={() => onOpenChange(false)} disabled={isLoading}>
        {t(confirmOperationMessages.cancel)}
      </AlertDialogCancel>
      <AlertDialogAction onclick={onContinue} disabled={isLoading}>
        {t(confirmOperationMessages.continue)}
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
