<script lang="ts">
  import { Tooltip as TooltipPrimitive } from 'bits-ui';

  type Props = TooltipPrimitive.RootProps & { delayDuration?: number };

  let { delayDuration = 0, children, ...rest }: Props = $props();
</script>

<!--
  The provider is folded into the root, as `shadcn/tooltip.tsx` already does:
  bits-ui throws when a `Tooltip.Root` has no provider above it, and nothing in
  this codebase groups tooltips to share a delay. `delayDuration = 0` keeps the
  React behaviour — these tooltips are the visible label of icon-only buttons,
  so bits-ui's 700 ms default would read as a bug.
-->
<TooltipPrimitive.Provider {delayDuration}>
  <TooltipPrimitive.Root {...rest} {children} />
</TooltipPrimitive.Provider>
