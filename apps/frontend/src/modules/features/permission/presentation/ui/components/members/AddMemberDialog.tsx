import { useEffect, useState, type ReactNode } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import {
  Button,
  Dialog,
  DialogContent,
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
import { UserPlus } from 'lucide-react';
import type { AssignableRole } from '@/modules/features/permission/presentation/hooks/use-assignable-roles.ts';
import { RoleChecklist } from '@/modules/features/permission/presentation/ui/components/members/RoleChecklist.tsx';

/** The little a candidate picker needs: both member lists already carry it. */
export interface MemberCandidate {
  userId: string;
  username: string;
}

interface AddMemberDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  description: ReactNode;
  /** People who may be admitted — never anyone already listed. */
  candidates: MemberCandidate[];
  /** Placeholder standing in for the picker when there is nobody to pick. */
  emptyCandidatesLabel: string;
  roles: AssignableRole[];
  rolesLabel: ReactNode;
  rolesLoading?: boolean;
  isPending: boolean;
  /** Pre-ticked on open: the role that admitting someone here means. */
  defaultRoleId?: string;
  /** Resolves to `true` once the member is in, which is what closes the dialog. */
  onSubmit: (userId: string, roleIds: string[]) => Promise<boolean>;
}

/**
 * Admits someone to a scope with the roles they should hold there.
 *
 * One form rather than two steps because admitting and granting are the same
 * act: membership is derived from grants, so a member with no role is not a
 * member at all. Multi-select on the roles for the same reason a member's
 * access is the sum of their roles — granting "developer" and "secrets reader"
 * together is the normal case, not two decisions.
 *
 * A dialog rather than a panel on the page: adding a member is occasional, and
 * the page's job is showing who is already there.
 */
export const AddMemberDialog = ({
  open,
  onOpenChange,
  title,
  description,
  candidates,
  emptyCandidatesLabel,
  roles,
  rolesLabel,
  rolesLoading = false,
  isPending,
  defaultRoleId,
  onSubmit,
}: AddMemberDialogProps) => {
  const { t } = useLingui();
  const [userId, setUserId] = useState('');
  const [selectedRoles, setSelectedRoles] = useState<Set<string>>(new Set());

  // Every opening starts from a clean form. The floor role comes pre-ticked
  // when the scope has one: it is what belonging means, and the rest of the
  // list is what you add on top of it.
  useEffect(() => {
    if (!open) return;
    setUserId('');
    setSelectedRoles(new Set(defaultRoleId ? [defaultRoleId] : []));
  }, [open, defaultRoleId]);

  const toggleRole = (roleId: string) =>
    setSelectedRoles(current => {
      const next = new Set(current);
      if (next.has(roleId)) next.delete(roleId);
      else next.add(roleId);
      return next;
    });

  const handleSubmit = async () => {
    if (userId === '' || selectedRoles.size === 0) return;
    if (await onSubmit(userId, [...selectedRoles])) onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='flex max-h-[85vh] flex-col sm:max-w-lg'>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        <div className='flex min-h-0 flex-col gap-4 overflow-y-auto pr-1'>
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='add-member-user'>
              <Trans>Member</Trans>
            </Label>
            <Select
              value={userId}
              disabled={isPending || candidates.length === 0}
              onValueChange={setUserId}
            >
              <SelectTrigger id='add-member-user' className='w-full'>
                <SelectValue
                  placeholder={candidates.length === 0 ? emptyCandidatesLabel : t`Select a user`}
                />
              </SelectTrigger>
              <SelectContent>
                {candidates.map(candidate => (
                  <SelectItem key={candidate.userId} value={candidate.userId}>
                    {candidate.username}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <RoleChecklist
            label={rolesLabel}
            roles={roles}
            isLoading={rolesLoading}
            disabled={isPending}
            selected={selectedRoles}
            onToggle={toggleRole}
          />
        </div>

        <DialogFooter>
          <Button
            type='button'
            variant='outline'
            disabled={isPending}
            onClick={() => onOpenChange(false)}
          >
            <Trans>Cancel</Trans>
          </Button>
          <Button
            type='button'
            disabled={userId === '' || selectedRoles.size === 0 || isPending}
            onClick={() => void handleSubmit()}
          >
            <UserPlus className='size-4' />
            <Trans>Add</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default AddMemberDialog;
