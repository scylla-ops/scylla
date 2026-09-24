<script lang="ts">
  import type { Snippet } from 'svelte';
  import ScyllaDialog from '../ScyllaDialog/ScyllaDialog.svelte';
  import SecretRevealChecklist from '../SecretRevealChecklist.svelte';

  interface Props {
    open: boolean;
    title: string;
    description: string;
    /** Blurred until revealed. */
    secret: string;
    /** E.g. the entity id. */
    secretLabel: string;
    copyToast?: string;
    secretStepTitle?: string;
    /** Shown once the secret is revealed, e.g. the run instructions. */
    secondStep?: { title: string; content: Snippet };
    /** Shown under the secret once revealed, when there is no second step. */
    revealedNote?: string;
    footerNote?: string;
    /** The user confirmed the copy. The caller closes the dialog. */
    onClose: () => void;
  }

  let { open, ...checklist }: Props = $props();
</script>

<!-- Cannot be dismissed: the checklist closes it once the secret was revealed. -->
<ScyllaDialog
  {open}
  onOpenChange={() => {}}
  dismissible={false}
  class="w-[calc(100vw-2rem)] gap-0 p-0 sm:max-w-lg"
>
  <SecretRevealChecklist {...checklist} />
</ScyllaDialog>
