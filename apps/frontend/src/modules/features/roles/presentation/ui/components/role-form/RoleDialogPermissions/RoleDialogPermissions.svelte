<script lang="ts">
  import { Badge, Checkbox, Label } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { Permission, PermissionScope } from '@platform/authz';
  import {
    getAlwaysGrantedPermissionsForScope,
    getEditablePermissionDefinitionsForScope,
    permissionLabelOf,
  } from '../../../../utils/permission-mapping.ts';
  import { buildPermissionTree } from '../../../../utils/permission-tree.ts';
  import { rolesMessages } from '../../../roles.messages.ts';
  import CheckboxTree from '../CheckboxTree.svelte';

  interface Props {
    scope: PermissionScope;
    permissions: Permission[];
    /** Permissions outside this build's catalog: kept on save, counted so nobody thinks they vanished. */
    preservedCount: number;
    /** What will be written, implicit permissions included. */
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

  // Against the role's scope: an organization role says "every project".
  const labelForScope = (permission: Permission) => permissionLabelOf(permission, scope);

  const nodes = $derived(
    buildPermissionTree(getEditablePermissionDefinitionsForScope(scope), labelForScope),
  );

  /** Conferred at this scope: shown ticked and locked. */
  const alwaysGranted = $derived(getAlwaysGrantedPermissionsForScope(scope));
</script>

<div class="flex flex-col gap-1.5">
  <div class="flex items-center justify-between">
    <Label>{t(rolesMessages.permissions)}</Label>
    <Badge variant="secondary">{t(rolesMessages.conferredCount(conferredCount))}</Badge>
  </div>

  <!-- Not `ScrollArea`: its root does not clip under a fixed height. -->
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

      <!-- Keyed on the scope: the tree seeds its checked set once. -->
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
