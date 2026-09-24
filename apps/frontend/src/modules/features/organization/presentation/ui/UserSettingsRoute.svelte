<script lang="ts">
  import { loadUserSettingsPage } from '@/modules/features/user';
  import OrganizationList from './OrganizationList/OrganizationList.svelte';

  interface Props {
    userId?: string;
  }

  let { userId }: Props = $props();
</script>

<!-- The page belongs to `user`; the panel is composed here, so the dependency stays organization → user. -->
{#await loadUserSettingsPage() then page}
  <page.default {userId} organizations={organizationsPanel} />
{/await}

{#snippet organizationsPanel()}
  <OrganizationList />
{/snippet}
