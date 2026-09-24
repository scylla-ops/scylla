<!--
  Shaped after `ConfirmOperationAlertDialog`, the real consumer: controlled
  `open`, a mutation behind Continue, and both buttons disabled while it runs.
-->
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
  } from './index.ts';

  interface Props {
    open?: boolean;
    isLoading?: boolean;
    onContinue?: () => void;
    onCancel?: () => void;
  }

  let { open = false, isLoading = false, onContinue, onCancel }: Props = $props();
</script>

<AlertDialog {open}>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
      <AlertDialogDescription>This action cannot be undone.</AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel onclick={onCancel} disabled={isLoading}>Cancel</AlertDialogCancel>
      <AlertDialogAction onclick={onContinue} disabled={isLoading}>Continue</AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
