<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Dialog, DialogContent } from '@shadcn';
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

  const block = (event: Event) => event.preventDefault();
</script>

<!--
  The one-time secret-reveal moment, as a quiet two-step checklist: copy the
  secret, then start the worker. One accent colour (primary), no warning
  banners — the "shown once" stake is carried by the copy, not by paint. The
  dialog cannot be dismissed until the secret has been revealed at least once.

  `{#key open}` is what replaces React's effect resetting the revealed flag: the
  checklist owns that state, so recreating it on every open resets it with
  nothing to keep in sync. The same rule `CLAUDE.md` states for React — reset
  state with a key, not an effect — reads identically here.
-->
<Dialog {open}>
  {#key open}
    <DialogContent
      class="[&>button]:hidden w-[calc(100vw-2rem)] sm:max-w-lg gap-0 p-0"
      onEscapeKeydown={block}
      onInteractOutside={block}
    >
      <SecretRevealChecklist {...checklist} />
    </DialogContent>
  {/key}
</Dialog>
