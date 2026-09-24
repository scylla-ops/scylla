<script lang="ts">
  import type { Snippet } from 'svelte';
  import ScyllaDialog from './ScyllaDialog.svelte';
  import SecretRevealChecklist from './SecretRevealChecklist.svelte';

  interface Props {
    open: boolean;
    title: string;
    description: string;
    /** The one-time secret value shown (blurred until revealed). */
    secret: string;
    /** Label displayed on the secret snippet (e.g. the entity id). */
    secretLabel: string;
    /** Toast shown after copying; defaults to "Secret copied". */
    copyToast?: string;
    /** Heading for the secret step; defaults to "Copy your secret". */
    secretStepTitle?: string;
    /**
     * Optional numbered second step, revealed once the secret is shown — e.g.
     * run instructions. When present, the checklist connector + step 2 bullet
     * appear.
     */
    secondStep?: { title: string; content: Snippet };
    /** Optional quiet note shown under the secret once revealed (no second step). */
    revealedNote?: string;
    /** Footer note; defaults to "You won't see this secret again." */
    footerNote?: string;
    /** Called once the user confirms they've copied it — caller closes + navigates. */
    onClose: () => void;
  }

  let { open, ...checklist }: Props = $props();
</script>

<!--
  The one-time secret reveal: copy the secret, then start the worker. It cannot
  be dismissed; the checklist closes it once the secret was revealed.
-->
<ScyllaDialog
  {open}
  onOpenChange={() => {}}
  dismissible={false}
  class="w-[calc(100vw-2rem)] gap-0 p-0 sm:max-w-lg"
>
  <SecretRevealChecklist {...checklist} />
</ScyllaDialog>
