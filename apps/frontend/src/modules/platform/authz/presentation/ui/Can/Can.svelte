<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { Permission } from '../../../domain/structs/permission.struct.ts';
  import type { PermissionTarget } from '../../../domain/entities/effective-permissions.entity.ts';
  import { can } from '../../authorization.ts';

  interface Props {
    permission: Permission;
    target?: PermissionTarget;
    fallback?: Snippet;
    children: Snippet;
  }

  let { permission, target, fallback, children }: Props = $props();

  const allowed = $derived(can(permission, target));
</script>

{#if allowed}
  {@render children()}
{:else}
  {@render fallback?.()}
{/if}
