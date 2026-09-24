<script lang="ts">
  import type { Snippet } from 'svelte';
  import Loader2Icon from '@lucide/svelte/icons/loader-2';
  import type { Permission } from '../../../domain/structs/permission.struct.ts';
  import type { PermissionTarget } from '../../../domain/entities/effective-permissions.entity.ts';
  import { authorizationReady, can } from '../../authorization.ts';
  import PermissionDenied from '../PermissionDenied/PermissionDenied.svelte';

  interface Props {
    permission: Permission;
    target?: PermissionTarget;
    message?: string;
    children: Snippet;
  }

  let { permission, target, message, children }: Props = $props();

  const ready = $derived(authorizationReady());
  const allowed = $derived(can(permission, target));
</script>

{#if !ready}
  <div class="flex h-full w-full items-center justify-center py-16">
    <Loader2Icon role="status" class="size-6 animate-spin text-muted-foreground" />
  </div>
{:else if allowed}
  {@render children()}
{:else}
  <PermissionDenied {message} />
{/if}
