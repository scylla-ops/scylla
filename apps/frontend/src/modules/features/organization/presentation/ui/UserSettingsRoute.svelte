<script lang="ts">
  import { loadUserSettingsPage } from '@/modules/features/user';
  import OrganizationList from './OrganizationList/OrganizationList.svelte';

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

  The page arrives through a loader, not a direct import: `layout` imports the
  `user` barrel eagerly, so a component re-exported from it would land in the
  entry chunk along with all of bits-ui.
-->
{#await loadUserSettingsPage() then page}
  <page.default {userId} organizations={organizationsPanel} />
{/await}

{#snippet organizationsPanel()}
  <OrganizationList />
{/snippet}
