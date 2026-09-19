<script lang="ts">
  import { Card } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { marketplaceMessages } from './marketplace.messages.ts';

  // Still a placeholder: the repository behind `marketplaceQueries` returns
  // hardcoded fixtures, so the catalog is not shown yet. `MarketplaceTopBar` and
  // `MarketItemList` are the screen this page becomes once the backend lands —
  // see the module's AGENTS.md.
</script>

<div class="flex h-full w-full items-center justify-center bg-background">
  <Card class="p-8 text-center">
    <p class="text-lg text-muted-foreground">{t(marketplaceMessages.comingSoon)}</p>
  </Card>
</div>
