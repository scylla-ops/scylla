<script lang="ts">
  import type { MarketItem } from '../../domain/structs/market-item.struct.ts';
  import { matchesFilter } from '../marketplace-filter.state.svelte.ts';
  import MarketItemCard from './MarketItemCard.svelte';

  interface Props {
    items: MarketItem[] | undefined;
    filter: string;
  }

  let { items, filter }: Props = $props();

  // Filtered here rather than inside the loop with an `{#if}`: the React version
  // returned an empty fragment for a non-match, which made every hidden item a
  // node in the flex row. Keying on the title also replaces the array index it
  // used, so re-filtering no longer reuses one card's state for another item.
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
