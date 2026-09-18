<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Button from './button.svelte';

  type Props = HTMLButtonAttributes & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<!--
  A plain button, not bits-ui's `AlertDialog.Action` — which closes the dialog
  on click. Every confirmation here drives a mutation and keeps the dialog open
  and disabled while it runs (`ConfirmOperationAlertDialog`'s `isLoading`), so
  closing on click would dismiss the dialog mid-request. The parent owns `open`.
-->
<Button class={className} {...rest}>
  {@render children?.()}
</Button>
