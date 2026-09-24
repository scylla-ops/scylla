<script lang="ts">
  import type { Snippet } from 'svelte';
  import InfoIcon from '@lucide/svelte/icons/info';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<!--
  The footnote under a member list, explaining what the list contains that the
  rows themselves cannot say — who is listed without holding a role here, and
  what the reader is not allowed to see.
-->
<p class="flex items-start gap-2 text-xs text-muted-foreground">
  <InfoIcon class="mt-0.5 size-3.5 shrink-0" />
  <span>{@render children()}</span>
</p>
