<script lang="ts">
  import { untrack, type Component } from 'svelte';
  import { RequirePermission } from '@platform/authz';
  import type { RouteParams } from '../declaration/scylla-module.struct.ts';
  import { currentMatch } from '../runtime/route-state.svelte.ts';
  import Redirect from './Redirect.svelte';

  const { route, params } = untrack(() => {
    const match = currentMatch();
    return { route: match?.route, params: { ...match?.params } };
  });
  const wrappers = route?.wrappers ?? [];
</script>

{#snippet content()}
  {#if route?.redirect !== undefined}
    <Redirect to={route.redirect} />
  {:else if route?.page}
    {#await route.page() then module}
      {@const Page = module.default as Component<RouteParams>}
      <Page {...params} />
    {/await}
  {/if}
{/snippet}

{#snippet guarded()}
  {#if route?.permission === undefined}
    {@render content()}
  {:else}
    <RequirePermission permission={route.permission}>
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
