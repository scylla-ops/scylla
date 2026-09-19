<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { userMessages } from '../user.messages.ts';
  import UserInformation from './UserInformation.svelte';

  interface Props {
    /** From the route: `/:organizationSlug/users/:userId`. */
    userId?: string;
    /**
     * The user's organizations panel, injected by whoever owns the route.
     * Listing organizations is the organization module's job, and it already
     * depends on `user` — so it fills this slot rather than being imported from
     * here. A snippet now, where it used to be a `ReactNode`.
     */
    organizations?: Snippet;
  }

  let { userId, organizations }: Props = $props();

  // Falling back to the signed-in user is what makes `/users/me`-style entry
  // work: the settings screen with no id in the route is your own.
  const shownUserId = $derived(userId ?? localStorage.getItem('userId') ?? undefined);
</script>

<!-- TODO: change and list only organizations that the user is in -->
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
