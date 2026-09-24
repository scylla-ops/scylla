<script lang="ts">
  import { Input } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { marketplaceFilter } from '../marketplace-filter.state.svelte.ts';
  import { marketplaceMessages } from './marketplace.messages.ts';
</script>

<Input
  value={marketplaceFilter.value}
  oninput={event => marketplaceFilter.set(event.currentTarget.value)}
  placeholder={t(marketplaceMessages.search)}
  class="w-[20%] bg-background"
/>
