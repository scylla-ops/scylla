<script lang="ts">
  import { contextStore } from '@platform/context';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import ProjectList from './ProjectList.svelte';
  import { projectMessages } from './project.messages.ts';

  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id);
</script>

{#if !organizationId}
  <div class="flex h-full items-center justify-center">
    <div class="space-y-2 text-center">
      <p class="text-lg font-semibold text-muted-foreground">
        {t(projectMessages.noOrganization)}
      </p>
      <p class="text-sm text-muted-foreground">{t(projectMessages.noOrganizationHint)}</p>
    </div>
  </div>
{:else}
  <!--
    Keyed on the organization, which is what replaces React's effect: switching
    organizations rebuilds the list and its pagination from scratch, so the page
    number cannot survive into an organization that may not have that many
    pages. `useProjects` used an effect calling `setPage(1)` for this.
  -->
  {#key organizationId}
    <ProjectList {organizationId} />
  {/key}
{/if}
