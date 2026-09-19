<script lang="ts">
  import { UserSettingsPage } from '@/modules/features/user';
  import OrganizationList from './OrganizationList.svelte';

  interface Props {
    /** From the route: `/:organizationSlug/users/:userId`. */
    userId?: string;
  }

  let { userId }: Props = $props();
</script>

<!--
  User settings with the organizations panel filled in.

  The page belongs to `user`; listing organizations belongs here. Composing on
  this side keeps the dependency one-way (organization → user) instead of the
  mutual import the panel would otherwise need. The slot is a snippet now, where
  it used to be a `ReactNode`.
-->
<UserSettingsPage {userId} organizations={organizationsPanel} />

{#snippet organizationsPanel()}
  <OrganizationList />
{/snippet}
