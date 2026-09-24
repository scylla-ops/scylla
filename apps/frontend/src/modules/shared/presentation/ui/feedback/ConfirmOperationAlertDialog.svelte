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
    isLoading?: boolean;
  }

  let { open, onOpenChange, onContinue, title, description, isLoading = false }: Props = $props();
</script>

<!-- The parent owns `open`: it keeps the dialog open and disabled until the mutation settles. -->
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
