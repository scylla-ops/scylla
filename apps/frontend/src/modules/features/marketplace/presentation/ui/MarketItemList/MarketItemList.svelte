<script lang="ts">
  import type { MarketItem } from '../../../domain/structs/market-item.struct.ts';
  import { matchesFilter } from '../../marketplace-filter.state.svelte.ts';
  import MarketItemCard from '../MarketItemCard.svelte';

  interface Props {
    items: MarketItem[] | undefined;
    filter: string;
  }

  let { items, filter }: Props = $props();

  // Keyed on the title, filtered before the loop.
  const visible = $derived((items ?? []).filter(item => matchesFilter(item, filter)));
</script>

<div class="flex h-fit flex-row flex-wrap gap-4">
  {#each visible as item (item.title)}
    <MarketItemCard
      class="h-50 max-w-[400px] min-w-[300px] flex-1"
      provider={item.provider}
      title={item.title}
      descrption={item.descrption}
    />
  {/each}
</div>
