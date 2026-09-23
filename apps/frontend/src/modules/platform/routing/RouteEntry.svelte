<script lang="ts">
  import { untrack, type Component } from 'svelte';
  import { RequirePermission } from '@platform/authz';
  import Redirect from './Redirect.svelte';
  import type { RouteParams } from './scylla-module.struct.ts';
  import {
    requiredPermission,
    routePage,
    routeParams,
    routeRedirect,
    routeWrappers,
  } from './route-state.ts';

  const { page, redirect, params, permission, wrappers } = untrack(() => ({
    page: routePage(),
    redirect: routeRedirect(),
    params: { ...routeParams() },
    permission: requiredPermission(),
    wrappers: routeWrappers(),
  }));
</script>

{#snippet content()}
  {#if redirect !== undefined}
    <Redirect to={redirect} />
  {:else if page}
    {#await page() then module}
      {@const Page = module.default as Component<RouteParams>}
      <Page {...params} />
    {/await}
  {/if}
{/snippet}

{#snippet guarded()}
  {#if permission === undefined}
    {@render content()}
  {:else}
    <RequirePermission {permission}>
      {@render content()}
    </RequirePermission>
  {/if}
{/snippet}

{#snippet wrapped(index: number)}
  {#if index < wrappers.length}
    {@const Wrapper = wrappers[index]}
    <Wrapper {params}>
      {@render wrapped(index + 1)}
    </Wrapper>
  {:else}
    {@render guarded()}
  {/if}
{/snippet}

{@render wrapped(0)}
