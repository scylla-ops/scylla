<script lang="ts">
  import UserPlusIcon from '@lucide/svelte/icons/user-plus';
  import {
    Button,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    Label,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { AssignableRole } from '../../assignable-roles.state.svelte.ts';
  import RoleChecklist from './RoleChecklist/RoleChecklist.svelte';
  import type { MemberCandidate } from './member-candidate.ts';
  import { membershipMessages } from '../membership.messages.ts';

  interface Props {
    title: string;
    description: string;
    candidates: MemberCandidate[];
    emptyCandidatesLabel: string;
    roles: AssignableRole[];
    rolesLabel: string;
    rolesLoading: boolean;
    isPending: boolean;
    defaultRoleId?: string;
    onSubmit: (userId: string, roleIds: string[]) => Promise<boolean>;
    onCancel: () => void;
  }

  let {
    title,
    description,
    candidates,
    emptyCandidatesLabel,
    roles,
    rolesLabel,
    rolesLoading,
    isPending,
    defaultRoleId,
    onSubmit,
    onCancel,
  }: Props = $props();

  // The floor role is pre-ticked when the scope has one.
  let userId = $state('');
  // svelte-ignore state_referenced_locally
  let selectedRoles = $state(new Set(defaultRoleId ? [defaultRoleId] : []));

  const toggleRole = (roleId: string) => {
    // Rebuilt whole by the `$derived` and never mutated after it is read, so a
    // reactive collection would only make a throwaway object track dependencies.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const next = new Set(selectedRoles);
    if (next.has(roleId)) next.delete(roleId);
    else next.add(roleId);
    selectedRoles = next;
  };

  const handleSubmit = async () => {
    if (userId === '' || selectedRoles.size === 0) return;
    await onSubmit(userId, [...selectedRoles]);
  };
</script>

<DialogHeader>
  <DialogTitle>{title}</DialogTitle>
  <DialogDescription>{description}</DialogDescription>
</DialogHeader>

<div class="flex min-h-0 flex-col gap-4 overflow-y-auto pr-1">
  <div class="flex flex-col gap-1.5">
    <Label for="add-member-user">{t(membershipMessages.member)}</Label>
    <Select
      type="single"
      value={userId}
      disabled={isPending || candidates.length === 0}
      onValueChange={value => (userId = value)}
    >
      <SelectTrigger id="add-member-user" class="w-full">
        <SelectValue
          placeholder={candidates.length === 0
            ? emptyCandidatesLabel
            : t(membershipMessages.selectAUser)}
        />
      </SelectTrigger>
      <SelectContent>
        {#each candidates as candidate (candidate.userId)}
          <SelectItem value={candidate.userId} label={candidate.username}>
            {candidate.username}
          </SelectItem>
        {/each}
      </SelectContent>
    </Select>
  </div>

  <RoleChecklist
    label={rolesLabel}
    {roles}
    isLoading={rolesLoading}
    disabled={isPending}
    selected={selectedRoles}
    onToggle={toggleRole}
  />
</div>

<DialogFooter>
  <Button type="button" variant="outline" disabled={isPending} onclick={onCancel}>
    {t(membershipMessages.cancel)}
  </Button>
  <Button
    type="button"
    disabled={userId === '' || selectedRoles.size === 0 || isPending}
    onclick={() => void handleSubmit()}
  >
    <UserPlusIcon class="size-4" />
    {t(membershipMessages.add)}
  </Button>
</DialogFooter>
