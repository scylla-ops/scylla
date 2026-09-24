<!--
  A tooltip cannot be rendered from a raw snippet — its parts are components —
  so the test drives it through here. `asChild` exercises the `child` snippet
  that replaces React's `asChild`, which is the arm `IconButton` will need.
-->
<script lang="ts">
  import { Button, Tooltip, TooltipContent, TooltipTrigger } from '../index.ts';

  interface Props {
    asChild?: boolean;
  }

  let { asChild = false }: Props = $props();
</script>

<Tooltip>
  {#if asChild}
    <TooltipTrigger>
      {#snippet child({ props })}
        <Button {...props}>Delete</Button>
      {/snippet}
    </TooltipTrigger>
  {:else}
    <TooltipTrigger>Delete</TooltipTrigger>
  {/if}
  <TooltipContent>Delete this secret</TooltipContent>
</Tooltip>
