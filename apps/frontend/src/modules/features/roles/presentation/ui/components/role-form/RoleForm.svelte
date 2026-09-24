<script lang="ts">
  import { type PermissionScope } from '@platform/authz';
  import {
    Button,
    DialogFooter,
    Input,
    Label,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { RoleEntity } from '../../../../domain/entities/role.entity.ts';
  import { createRoleForm, type AccessKind } from '../../../role-form.state.svelte.ts';
  import { ALL_SCOPES, scopeLabelOf } from '../../../utils/permission-mapping.ts';
  import { rolesMessages } from '../../roles.messages.ts';
  import RoleDialogPermissions from './RoleDialogPermissions/RoleDialogPermissions.svelte';

  interface Props {
    /** `null` when creating. */
    role: RoleEntity | null;
    onDone: () => void;
  }

  let { role, onDone }: Props = $props();

  // Seeded once, deliberately: `RoleFormDialog` renders this under `{#key open}`,
  // so a new opening builds a new component rather than re-reading the prop.
  // svelte-ignore state_referenced_locally
  const form = createRoleForm(role);

  const submit = async () => {
    if (!form.isValid) return;
    if (await form.submit()) onDone();
  };
</script>

<div class="flex flex-col gap-4 overflow-y-auto pr-1">
  <div class="flex flex-col gap-1.5">
    <Label for="role-name">{t(rolesMessages.name)}</Label>
    <Input
      id="role-name"
      autofocus
      bind:value={form.name}
      disabled={form.isPending}
      placeholder={t(rolesMessages.namePlaceholder)}
    />
  </div>

  <div class="flex flex-col gap-1.5">
    <Label for="role-description">{t(rolesMessages.description)}</Label>
    <Input
      id="role-description"
      bind:value={form.description}
      disabled={form.isPending}
      placeholder={t(rolesMessages.descriptionPlaceholder)}
    />
  </div>

  <div class="flex flex-col gap-1.5">
    <Label for="role-scope">{t(rolesMessages.scope)}</Label>
    <Select
      type="single"
      value={String(form.scope)}
      disabled={form.isEdit || form.isPending}
      onValueChange={value => form.changeScope(Number(value) as PermissionScope)}
    >
      <SelectTrigger id="role-scope" class="w-full">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {#each ALL_SCOPES as scope (scope)}
          <SelectItem value={String(scope)} label={scopeLabelOf(scope)}>
            {scopeLabelOf(scope)}
          </SelectItem>
        {/each}
      </SelectContent>
    </Select>
    {#if form.isEdit}
      <p class="text-xs text-muted-foreground">{t(rolesMessages.scopeIsFixed)}</p>
    {/if}
  </div>

  <div class="flex flex-col gap-1.5">
    <Label for="role-access">{t(rolesMessages.access)}</Label>
    <Select
      type="single"
      value={form.accessKind}
      disabled={form.isPending}
      onValueChange={value => (form.accessKind = value as AccessKind)}
    >
      <SelectTrigger id="role-access" class="w-full">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="fullControl" label={t(rolesMessages.fullControl)}>
          {t(rolesMessages.fullControl)}
        </SelectItem>
        <SelectItem value="restricted" label={t(rolesMessages.restricted)}>
          {t(rolesMessages.restricted)}
        </SelectItem>
      </SelectContent>
    </Select>
  </div>

  {#if form.accessKind === 'restricted'}
    <RoleDialogPermissions
      scope={form.scope}
      permissions={form.permissions}
      preservedCount={form.preservedCount}
      conferredCount={form.conferredCount}
      isPending={form.isPending}
      onPermissionsChange={next => (form.permissions = next)}
    />
  {/if}
</div>

<DialogFooter>
  <Button type="button" variant="outline" disabled={form.isPending} onclick={onDone}>
    {t(rolesMessages.cancel)}
  </Button>
  <Button
    type="button"
    disabled={!form.isValid || form.isPending}
    onclick={() => void submit()}
  >
    {form.isEdit ? t(rolesMessages.saveChanges) : t(rolesMessages.createRole)}
  </Button>
</DialogFooter>
