<script lang="ts">
  import { Card } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { marketplaceMessages } from './marketplace.messages.ts';

  // A placeholder: the repository returns fixtures. See AGENTS.md.
</script>

<div class="flex h-full w-full items-center justify-center bg-background">
  <Card class="p-8 text-center">
    <p class="text-lg text-muted-foreground">{t(marketplaceMessages.comingSoon)}</p>
  </Card>
</div>
