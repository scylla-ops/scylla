<!--
  Phase 0's proof of life, and deliberately in `core/`.

  It reads one thing from each singleton Phase 0 established, so the test beside
  it fails the day any of them goes back behind a React provider. It lives in the
  composition root rather than next to `SvelteIsland` because it reaches into
  `platform/`, and `shared/` — where the bridge itself belongs — sits below
  platform and may not. `depcruise`'s `shared-is-generic` catches that, on
  `.svelte` files too.
-->
<script lang="ts">
  import { Permission, can } from '@platform/authz';
  import { useContextStore } from '@platform/context';
  import { getModuleDomain } from '@platform/di';
  import { toSvelteStore } from '@shared/presentation/stores/to-svelte-store.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { platformSingletonsMessages } from './platform-singletons.messages.ts';

  interface Props {
    /** Pushed by the React side, to prove props survive a parent re-render. */
    label: string;
  }

  const { label }: Props = $props();

  const organizationName = toSvelteStore(useContextStore, state => state.organization.name);

  const allowed = can(Permission.LIST_SECRETS);
  const domain = getModuleDomain<{ marker: string }>('island-fixture');
</script>

<section>
  <p data-testid="greeting">{t(platformSingletonsMessages.greeting)}</p>
  <p data-testid="label">{label}</p>
  <p data-testid="organization">{$organizationName ?? 'none'}</p>
  <p data-testid="permission">{allowed ? 'allowed' : 'denied'}</p>
  <p data-testid="domain">{domain.marker}</p>
</section>
