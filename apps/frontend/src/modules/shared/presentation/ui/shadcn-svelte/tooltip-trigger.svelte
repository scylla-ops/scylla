<script lang="ts">
  import { Tooltip as TooltipPrimitive } from 'bits-ui';

  let { ...rest }: TooltipPrimitive.TriggerProps = $props();
</script>

<!--
  React's `asChild` is bits-ui's `child` snippet, and it rides through `rest`.
  A caller that needs the trigger to *be* its button writes:

    <TooltipTrigger>
      {#snippet child({ props })}
        <Button {...props}>…</Button>
      {/snippet}
    </TooltipTrigger>
-->
<TooltipPrimitive.Trigger data-slot="tooltip-trigger" {...rest} />
