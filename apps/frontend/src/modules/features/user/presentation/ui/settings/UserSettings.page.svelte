<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { userMessages } from '../user.messages.ts';
  import UserInformation from './UserInformation.svelte';

  interface Props {
    userId?: string;
    /** Filled by the route's owner (`organization`), which already depends on `user`. */
    organizations?: Snippet;
  }

  let { userId, organizations }: Props = $props();

  // No id in the route: your own settings.
  const shownUserId = $derived(userId ?? localStorage.getItem('userId') ?? undefined);
</script>

<!-- TODO: list only the organizations the user is in. -->
<div class="flex w-full flex-col gap-4">
  <div class="flex items-center gap-4">
    <h1 class="text-3xl font-bold">{t(userMessages.userSettings)}</h1>
  </div>

  <div class="flex space-x-6 bg-background">
    <div class="w-1/2">
      <UserInformation userId={shownUserId} />
    </div>

    {#if organizations}
      <div class="w-1/2">
        <Card class="w-full">
          <CardHeader>
            <CardTitle>{t(userMessages.organizations)}</CardTitle>
          </CardHeader>
          <CardContent class="space-y-4">{@render organizations()}</CardContent>
        </Card>
      </div>
    {/if}
  </div>
</div>
