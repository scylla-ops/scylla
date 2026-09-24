<script lang="ts">
  import { ScyllaDialog } from '@shared/presentation/ui';
  import type { AssignableRole } from '../../assignable-roles.state.svelte.ts';
  import AddMemberForm from './AddMemberForm.svelte';
  import type { MemberCandidate } from './member-candidate.ts';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    title: string;
    description: string;
    /** People who may be admitted — never anyone already listed. */
    candidates: MemberCandidate[];
    /** Placeholder standing in for the picker when there is nobody to pick. */
    emptyCandidatesLabel: string;
    roles: AssignableRole[];
    rolesLabel: string;
    rolesLoading?: boolean;
    isPending: boolean;
    /** Pre-ticked on open: the role that admitting someone here means. */
    defaultRoleId?: string;
    /** Resolves to `true` once the member is in, which is what closes the dialog. */
    onSubmit: (userId: string, roleIds: string[]) => Promise<boolean>;
  }

  let { open, onOpenChange, rolesLoading = false, onSubmit, ...form }: Props = $props();

  const submit = async (userId: string, roleIds: string[]) => {
    const added = await onSubmit(userId, roleIds);
    if (added) onOpenChange(false);
    return added;
  };
</script>

<ScyllaDialog {open} {onOpenChange} class="flex max-h-[85vh] flex-col sm:max-w-lg">
  <AddMemberForm {...form} {rolesLoading} onSubmit={submit} onCancel={() => onOpenChange(false)} />
</ScyllaDialog>
