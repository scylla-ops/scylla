<script lang="ts">
  import { Dialog, DialogContent } from '@shadcn-svelte';
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

<!--
  Admits someone to a scope with the roles they should hold there.

  One form rather than two steps because admitting and granting are the same act:
  membership is derived from grants, so a member with no role is not a member at
  all. Multi-select on the roles for the same reason a member's access is the sum
  of their roles — granting "developer" and "secrets reader" together is the
  normal case, not two decisions.

  A dialog rather than a panel on the page: adding a member is occasional, and
  the page's job is showing who is already there.

  `{#key open}` is what resets the form: the state lives in `AddMemberForm`, so
  recreating it clears the selection with no effect watching `open` — the same
  rule `CLAUDE.md` gives React, spelled the same way here.
-->
<Dialog {open} {onOpenChange}>
  {#key open}
    <DialogContent class="flex max-h-[85vh] flex-col sm:max-w-lg">
      <AddMemberForm
        {...form}
        {rolesLoading}
        onSubmit={submit}
        onCancel={() => onOpenChange(false)}
      />
    </DialogContent>
  {/key}
</Dialog>
