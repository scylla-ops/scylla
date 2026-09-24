<script lang="ts">
  import GlobeIcon from '@lucide/svelte/icons/globe';
  import InfoIcon from '@lucide/svelte/icons/info';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import { can, Permission, PermissionScope } from '@platform/authz';
  import {
    Badge,
    Button,
    DialogFooter,
    Label,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@shadcn';
  import { GatedButton, ScyllaDialog } from '@shared/presentation/ui';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { RoleEntity } from '../../../../domain/entities/role.entity.ts';
  import { createGrantCreator } from '../../../grant-creator.state.svelte.ts';
  import { rolesMessages } from '../../roles.messages.ts';
  import TargetChecklist from './TargetChecklist.svelte';

  interface Props {
    role: RoleEntity;
  }

  let { role }: Props = $props();

  let open = $state(false);

  const creator = createGrantCreator(() => role);

  const userPlaceholder = $derived(
    creator.isProjectScope && !creator.browseOrgId
      ? t(rolesMessages.pickAnOrganizationFirst)
      : t(rolesMessages.selectAUser),
  );

  const ineligibleReason = (reason: 'not-admitted' | 'cannot-see-projects') =>
    reason === 'not-admitted' ? t(rolesMessages.notAMember) : t(rolesMessages.cannotSeeProjects);

  const openDialog = () => {
    creator.reset();
    open = true;
  };

  const submit = async () => {
    const created = await creator.submit();
    if (created === null) return; // already toasted by the mutation cache

    toast.success(
      created > 1 ? t(rolesMessages.grantsCreated(created)) : t(rolesMessages.grantCreated),
    );
    open = false;
  };
</script>

<!-- Users not admitted to the organization are greyed out: the backend would reject a project grant. -->
{#snippet userPicker()}
  <div class="flex flex-col gap-1.5">
    <Label for="grant-user">{t(rolesMessages.user)}</Label>
    <Select
      type="single"
      value={creator.userId}
      disabled={creator.isPending || (creator.isProjectScope && !creator.browseOrgId)}
      onValueChange={value => (creator.userId = value)}
    >
      <SelectTrigger id="grant-user" class="w-full">
        <SelectValue placeholder={userPlaceholder} />
      </SelectTrigger>
      <SelectContent>
        <!-- A disabled item gets no hover: the reason is shown inline. -->
        {#each creator.users as user (user.id)}
          <SelectItem value={user.id} label={user.name} disabled={!!user.ineligible}>
            <span class="flex w-full items-center gap-2">
              <span class="truncate">{user.name}</span>
              {#if user.ineligible}
                <span class="ml-auto shrink-0 text-xs text-muted-foreground italic">
                  {ineligibleReason(user.ineligible)}
                </span>
              {/if}
            </span>
          </SelectItem>
        {/each}
      </SelectContent>
    </Select>
  </div>
{/snippet}

<GatedButton
  allowed={can(Permission.MANAGE_SYSTEM_GRANTS)}
  deniedReason={t(rolesMessages.grantDenied)}
  size="sm"
  onclick={openDialog}
>
  <PlusIcon class="size-4" />
  {t(rolesMessages.addGrant)}
</GatedButton>

<ScyllaDialog
  {open}
  onOpenChange={next => (open = next)}
  class="flex max-h-[85vh] max-w-lg flex-col"
  title={t(rolesMessages.grantTitle(role.name))}
  description={t(rolesMessages.grantSubtitle)}
>

  <div class="flex flex-col gap-4 overflow-y-auto pr-1">
    {#if creator.scope === PermissionScope.SYSTEM}
      {@render userPicker()}
      <div
        class="flex items-center gap-2 rounded-lg border px-3 py-2.5 text-sm text-muted-foreground"
      >
        <GlobeIcon class="size-4 shrink-0" />
        {t(rolesMessages.systemWide)}
      </div>
    {:else if creator.scope === PermissionScope.ORGANIZATION}
      {@render userPicker()}
      <TargetChecklist
        label={t(rolesMessages.organizations)}
        empty={t(rolesMessages.noOrganizations)}
        isLoading={creator.organizationsLoading}
        options={creator.organizations}
        disabled={creator.isPending}
        isSelected={id => creator.isSelected(id)}
        isGranted={id => creator.isAlreadyGranted(id)}
        onToggle={option => creator.toggle(option)}
      />
    {:else}
      <div class="flex flex-col gap-4">
        <div class="flex flex-col gap-1.5">
          <Label for="grant-org">{t(rolesMessages.organization)}</Label>
          <Select
            type="single"
            value={creator.browseOrgId ?? ''}
            disabled={creator.isPending || creator.organizationsLoading}
            onValueChange={value => creator.browseOrganization(value)}
          >
            <SelectTrigger id="grant-org" class="w-full">
              <SelectValue placeholder={t(rolesMessages.pickAnOrganization)} />
            </SelectTrigger>
            <SelectContent>
              {#each creator.organizations as organization (organization.id)}
                <SelectItem value={organization.id} label={organization.name}>
                  {organization.name}
                </SelectItem>
              {/each}
            </SelectContent>
          </Select>
        </div>

        {@render userPicker()}

        {#if creator.browseOrgId && !creator.hasSelectableUser}
          <div
            class="flex items-start gap-2 rounded-lg border px-3 py-2.5 text-sm text-muted-foreground"
          >
            <InfoIcon class="mt-0.5 size-4 shrink-0" />
            {t(rolesMessages.nobodyEligible)}
          </div>
        {/if}

        {#if creator.browseOrgId && creator.hasSelectableUser}
          <TargetChecklist
            label={t(rolesMessages.projects)}
            empty={t(rolesMessages.noProjects)}
            isLoading={creator.projectsLoading}
            options={creator.projects}
            disabled={creator.isPending}
            isSelected={id => creator.isSelected(id)}
            isGranted={id => creator.isAlreadyGranted(id)}
            onToggle={option => creator.toggle(option)}
          />
        {/if}
      </div>
    {/if}

    {#if creator.needsTargets && creator.selectedCount > 0}
      <div class="flex flex-col gap-1.5">
        <Label>{t(rolesMessages.selectedCount(creator.selectedCount))}</Label>
        <div class="flex flex-wrap gap-1.5">
          {#each creator.selected as option (option.id)}
            <Badge variant="secondary" class="gap-1 pr-1">
              {option.name}
              <Button
                variant="outline"
                size="xs"
                type="button"
                disabled={creator.isPending}
                aria-label={option.name}
                class="rounded hover:text-destructive"
                onclick={() => creator.toggle(option)}
              >
                ×
              </Button>
            </Badge>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <DialogFooter>
    <Button
      type="button"
      variant="outline"
      disabled={creator.isPending}
      onclick={() => (open = false)}
    >
      {t(rolesMessages.cancel)}
    </Button>
    <Button
      type="button"
      disabled={!creator.isValid || creator.isPending}
      onclick={() => void submit()}
    >
      {t(rolesMessages.createGrant)}
    </Button>
  </DialogFooter>
</ScyllaDialog>
