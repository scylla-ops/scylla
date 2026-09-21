<script lang="ts">
  import { Badge, Checkbox, Label } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { Permission, PermissionScope } from '@platform/authz';
  import {
    getAlwaysGrantedPermissionsForScope,
    getEditablePermissionDefinitionsForScope,
    permissionLabelOf,
  } from '../../../utils/permission-mapping.ts';
  import { buildPermissionTree } from '../../../utils/permission-tree.ts';
  import { rolesMessages } from '../../roles.messages.ts';
  import CheckboxTree from './CheckboxTree.svelte';

  interface Props {
    scope: PermissionScope;
    permissions: Permission[];
    /**
     * How many permissions the role holds outside this build's catalog. They
     * are kept on save; the count is shown so nobody thinks they vanished.
     */
    preservedCount: number;
    /** What will actually be written — riders included. The honest count. */
    conferredCount: number;
    isPending: boolean;
    onPermissionsChange: (permissions: Permission[]) => void;
  }

  let {
    scope,
    permissions,
    preservedCount,
    conferredCount,
    isPending,
    onPermissionsChange,
  }: Props = $props();

  // Labels read against the *role's* scope, so a project permission conferred by
  // an organization role says "every project" rather than "the project".
  const labelForScope = (permission: Permission) => permissionLabelOf(permission, scope);

  const nodes = $derived(
    buildPermissionTree(getEditablePermissionDefinitionsForScope(scope), labelForScope),
  );

  /**
   * Conferred by construction at this scope, so shown ticked and locked rather
   * than hidden: the reader still learns the role carries it, and nobody can
   * build a role that admits someone to a place they cannot see.
   */
  const alwaysGranted = $derived(getAlwaysGrantedPermissionsForScope(scope));
</script>

<div class="flex flex-col gap-1.5">
  <div class="flex items-center justify-between">
    <Label>{t(rolesMessages.permissions)}</Label>
    <Badge variant="secondary">{t(rolesMessages.conferredCount(conferredCount))}</Badge>
  </div>

  <!-- Plain overflow rather than the shared `ScrollArea` — that root is only
       `relative`, so a fixed height on it clips nothing. -->
  <div class="h-56 overflow-y-auto rounded-lg border p-2">
    <div class="flex flex-col gap-0.5">
      {#each alwaysGranted as permission (permission)}
        <label
          class="flex cursor-not-allowed items-center gap-2 rounded-md px-2 py-1.5"
          aria-disabled="true"
        >
          <Checkbox checked disabled aria-label={labelForScope(permission)} />
          <span class="text-sm">{labelForScope(permission)}</span>
          <Badge variant="outline" class="ml-auto text-[10px]">{t(rolesMessages.always)}</Badge>
        </label>
      {/each}

      <!--
        Keyed on the scope: switching it rebuilds the tree from another catalog
        slice, and the component seeds its checked set once, at construction.
      -->
      {#key scope}
        <CheckboxTree
          {nodes}
          checkedIds={permissions}
          allDisabled={isPending}
          onCheckedChange={onPermissionsChange}
        />
      {/key}
    </div>
  </div>

  {#if alwaysGranted.length > 0}
    <p class="text-xs text-muted-foreground">{t(rolesMessages.alwaysGrantedNote)}</p>
  {/if}
  {#if preservedCount > 0}
    <p class="text-xs text-muted-foreground">{t(rolesMessages.preservedNote(preservedCount))}</p>
  {/if}
</div>
