import { useEffect, useMemo, useState, type ReactNode } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { toast } from '@shared/presentation/utils/toast.ts';
import {
  Badge,
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
import { Checkbox } from '@shadcn/checkbox.tsx';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { Globe, Info, Plus, X } from 'lucide-react';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import type { RoleEntity } from '@/modules/features/permission/domain/entities/role.entity.ts';
import {
  Permission,
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { PermissionButton } from '@/modules/features/permission/presentation/ui/authorization/PermissionButton.tsx';
import { useGrants } from '@/modules/features/permission/presentation/hooks/use-grants.ts';
import { useProjectGrantEligibility } from '@/modules/features/permission/presentation/hooks/use-project-grant-eligibility.ts';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { useProjects } from '@/modules/features/project/presentation/hooks/useProjects.ts';

interface GrantCreatorProps {
  role: RoleEntity;
}

/** A selectable grant target: the scope id (org/project) plus its display name. */
interface TargetOption {
  id: string;
  name: string;
}

/** A user offered in the picker; `disabledReason` greys it out and says why. */
interface UserOption {
  id: string;
  name: string;
  disabledReason?: ReactNode;
}

/**
 * Grants a role to a user within the scope the role requires:
 * - SYSTEM       → the user, system-wide (no scope id).
 * - ORGANIZATION → the user, on one or more organizations.
 * - PROJECT      → the user, on one or more projects of a chosen organization.
 *
 * One grant is created per selected scope target.
 *
 * Project grants follow the backend's tenant boundary: a user may only receive
 * one once the organization owning the project has already admitted them, i.e.
 * they hold a grant bound to that organization — which is exactly what being a
 * member of it means. The picker therefore asks for the organization first and
 * offers only the users it has admitted, turning a server-side rejection into a
 * constraint you can see. Admitting someone is the members dialog's job, in the
 * organization switcher.
 */
export const GrantCreator = ({ role }: GrantCreatorProps) => {
  const { t } = useLingui();
  const [open, setOpen] = useState(false);

  const { grants, createGrant } = useGrants();
  const { users } = useUsers();

  const [userId, setUserId] = useState('');
  // Chosen scope targets, keyed by id → display name (accumulates across orgs).
  const [selected, setSelected] = useState<Map<string, string>>(new Map());
  // For PROJECT scope: which org's projects are currently being browsed.
  const [browseOrgId, setBrowseOrgId] = useState<string | null>(null);

  // Reset the form each time the dialog opens.
  useEffect(() => {
    if (!open) return;
    setUserId('');
    setSelected(new Map());
    setBrowseOrgId(null);
  }, [open]);

  const isProjectScope = role.scope === PermissionScope.PROJECT;

  const userItems: UserOption[] = useMemo(
    () => (users?.items ?? []).map(user => ({ id: user.userId, name: user.username })),
    [users],
  );

  const { eligibilityFor } = useProjectGrantEligibility(browseOrgId);

  /**
   * Everyone stays in the list; those who can't receive this grant are greyed
   * out with the reason, so the constraint is visible rather than a silently
   * shorter list.
   */
  const selectableUsers: UserOption[] = useMemo(() => {
    if (!isProjectScope || !browseOrgId) return userItems;
    return userItems.map(user => {
      switch (eligibilityFor(user.id)) {
        case 'not-admitted':
          return { ...user, disabledReason: <Trans>not a member of this organization</Trans> };
        case 'cannot-see-projects':
          return {
            ...user,
            disabledReason: <Trans>can't see this organization's projects</Trans>,
          };
        default:
          return user;
      }
    });
  }, [isProjectScope, browseOrgId, userItems, eligibilityFor]);

  const hasSelectableUser = selectableUsers.some(user => !user.disabledReason);

  // Switching organization changes who is eligible — drop a now-invalid pick.
  useEffect(() => {
    if (!isProjectScope || userId === '') return;
    if (selectableUsers.some(user => user.id === userId && user.disabledReason)) setUserId('');
  }, [isProjectScope, selectableUsers, userId]);

  // Scope ids where this user already holds this role — offered as disabled.
  const alreadyGranted = useMemo(() => {
    const ids = new Set<string>();
    if (!userId) return ids;
    for (const grant of grants) {
      if (
        grant.roleId === role.id &&
        grant.principal.kind === PrincipalKind.USER &&
        grant.principal.id === userId
      ) {
        ids.add(grant.scopeId);
      }
    }
    return ids;
  }, [grants, role.id, userId]);

  const toggle = (option: TargetOption) =>
    setSelected(prev => {
      const next = new Map(prev);
      if (next.has(option.id)) next.delete(option.id);
      else next.set(option.id, option.name);
      return next;
    });

  const needsTargets = role.scope !== PermissionScope.SYSTEM;
  const isValid = userId !== '' && (!needsTargets || selected.size > 0);
  const isPending = createGrant.isPending;

  const handleSubmit = async () => {
    if (!isValid) return;
    const scopeIds = role.scope === PermissionScope.SYSTEM ? [''] : [...selected.keys()];
    try {
      await Promise.all(
        scopeIds.map(scopeId =>
          createGrant.mutateAsync({
            principal: { kind: PrincipalKind.USER, id: userId },
            roleId: role.id,
            scope: role.scope,
            scopeId,
          }),
        ),
      );
      toast.success(scopeIds.length > 1 ? t`${scopeIds.length} grants created` : t`Grant created`);
      setOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t`Failed to create grant`);
    }
  };

  const userPicker = (
    <UserPicker
      value={userId}
      onChange={setUserId}
      options={selectableUsers}
      disabled={isPending || (isProjectScope && !browseOrgId)}
      placeholder={
        isProjectScope && !browseOrgId ? t`Pick an organization first` : t`Select a user`
      }
    />
  );

  return (
    <>
      {/* Grants are administered system-wide in this build — one permission for
          every scope, held by system administrators only. */}
      <PermissionButton
        permission={Permission.MANAGE_SYSTEM_GRANTS}
        size='sm'
        onClick={() => setOpen(true)}
        deniedReason={<Trans>You don't have permission to grant this role.</Trans>}
      >
        <Plus className='size-4' />
        <Trans>Add grant</Trans>
      </PermissionButton>

      <Dialog open={open} onOpenChange={value => !value && setOpen(false)}>
        <DialogContent className='max-w-lg flex flex-col max-h-[85vh]'>
          <DialogHeader className='space-y-3'>
            <DialogTitle className='text-lg font-semibold'>
              <Trans>Grant “{role.name}”</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Choose who receives this role and where it applies.</Trans>
            </DialogDescription>
          </DialogHeader>

          <div className='flex flex-col gap-4 overflow-y-auto pr-1'>
            {role.scope === PermissionScope.SYSTEM && (
              <>
                {userPicker}
                <div className='flex items-center gap-2 rounded-lg border px-3 py-2.5 text-sm text-muted-foreground'>
                  <Globe className='size-4 shrink-0' />
                  <Trans>This role grants access across the whole system.</Trans>
                </div>
              </>
            )}

            {role.scope === PermissionScope.ORGANIZATION && (
              <>
                {userPicker}
                <OrganizationTargets
                  disabled={isPending}
                  selected={selected}
                  alreadyGranted={alreadyGranted}
                  onToggle={toggle}
                />
              </>
            )}

            {isProjectScope && (
              <ProjectScopeFields
                disabled={isPending}
                browseOrgId={browseOrgId}
                onBrowseOrg={setBrowseOrgId}
                userPicker={userPicker}
                hasSelectableUser={hasSelectableUser}
                selected={selected}
                alreadyGranted={alreadyGranted}
                onToggle={toggle}
              />
            )}

            {needsTargets && selected.size > 0 && (
              <div className='flex flex-col gap-1.5'>
                <Label>
                  <Trans>Selected ({selected.size})</Trans>
                </Label>
                <div className='flex flex-wrap gap-1.5'>
                  {[...selected.entries()].map(([id, name]) => (
                    <Badge key={id} variant='secondary' className='gap-1 pr-1'>
                      {name}
                      <Button
                        variant={'outline'}
                        size={'xs'}
                        type='button'
                        disabled={isPending}
                        className='rounded hover:text-destructive'
                        onClick={() => toggle({ id, name })}
                      >
                        <X className='size-3' />
                      </Button>
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </div>

          <DialogFooter>
            <Button
              type='button'
              variant='outline'
              disabled={isPending}
              onClick={() => setOpen(false)}
            >
              <Trans>Cancel</Trans>
            </Button>
            <Button type='button' disabled={!isValid || isPending} onClick={handleSubmit}>
              <Trans>Create grant</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
};

// ── User picker ───────────────────────────────────────────────────────────────

interface UserPickerProps {
  value: string;
  onChange: (userId: string) => void;
  options: UserOption[];
  disabled: boolean;
  placeholder: string;
}

/**
 * A disabled `SelectItem` carries `pointer-events-none`, so a hover tooltip
 * would never fire on it — the reason is rendered inline instead, which also
 * saves the reader a hover to find out why a name is greyed out.
 */
const UserPicker = ({ value, onChange, options, disabled, placeholder }: UserPickerProps) => (
  <div className='flex flex-col gap-1.5'>
    <Label htmlFor='grant-user'>
      <Trans>User</Trans>
    </Label>
    <Select value={value} disabled={disabled} onValueChange={onChange}>
      <SelectTrigger id='grant-user' className='w-full'>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        {options.map(user => (
          <SelectItem key={user.id} value={user.id} disabled={!!user.disabledReason}>
            <span className='flex w-full items-center gap-2'>
              <span className='truncate'>{user.name}</span>
              {user.disabledReason && (
                <span className='ml-auto shrink-0 text-xs text-muted-foreground italic'>
                  {user.disabledReason}
                </span>
              )}
            </span>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  </div>
);

// ── Organization scope ────────────────────────────────────────────────────────

interface TargetsProps {
  disabled: boolean;
  selected: Map<string, string>;
  alreadyGranted: Set<string>;
  onToggle: (option: TargetOption) => void;
}

const OrganizationTargets = ({ disabled, selected, alreadyGranted, onToggle }: TargetsProps) => {
  const { organizations, isLoading } = useOrganizations();
  const options: TargetOption[] = (organizations ?? []).map(org => ({
    id: idValue(org.organizationId),
    name: org.name,
  }));

  return (
    <TargetChecklist
      label={<Trans>Organizations</Trans>}
      empty={<Trans>No organizations available.</Trans>}
      isLoading={isLoading}
      options={options}
      disabled={disabled}
      selected={selected}
      alreadyGranted={alreadyGranted}
      onToggle={onToggle}
    />
  );
};

// ── Project scope ─────────────────────────────────────────────────────────────

interface ProjectScopeFieldsProps extends TargetsProps {
  browseOrgId: string | null;
  onBrowseOrg: (orgId: string) => void;
  /** Rendered between the organization and the project list. */
  userPicker: ReactNode;
  hasSelectableUser: boolean;
}

/**
 * Organization → user → projects, in that order: who may receive a project
 * grant depends on the organization that owns the project.
 */
const ProjectScopeFields = ({
  disabled,
  browseOrgId,
  onBrowseOrg,
  userPicker,
  hasSelectableUser,
  selected,
  alreadyGranted,
  onToggle,
}: ProjectScopeFieldsProps) => {
  const { organizations, isLoading: orgsLoading } = useOrganizations();
  const { projects, isLoading: projectsLoading } = useProjects(browseOrgId);

  const options: TargetOption[] = (projects ?? []).map(project => ({
    id: project.id,
    name: project.name,
  }));

  return (
    <div className='flex flex-col gap-4'>
      <div className='flex flex-col gap-1.5'>
        <Label htmlFor='grant-org'>
          <Trans>Organization</Trans>
        </Label>
        <Select
          value={browseOrgId ?? ''}
          disabled={disabled || orgsLoading}
          onValueChange={onBrowseOrg}
        >
          <SelectTrigger id='grant-org' className='w-full'>
            <SelectValue placeholder={<Trans>Pick an organization</Trans>} />
          </SelectTrigger>
          <SelectContent>
            {(organizations ?? []).map(org => (
              <SelectItem key={idValue(org.organizationId)} value={idValue(org.organizationId)}>
                {org.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {userPicker}

      {browseOrgId && !hasSelectableUser && (
        <div className='flex items-start gap-2 rounded-lg border px-3 py-2.5 text-sm text-muted-foreground'>
          <Info className='size-4 shrink-0 mt-0.5' />
          <Trans>
            Nobody can receive a project grant here yet. Admit someone to the organization first —
            from the organization switcher, under “Members” — and they will show up here.
          </Trans>
        </div>
      )}

      {browseOrgId && hasSelectableUser && (
        <TargetChecklist
          label={<Trans>Projects</Trans>}
          empty={<Trans>No projects in this organization.</Trans>}
          isLoading={projectsLoading}
          options={options}
          disabled={disabled}
          selected={selected}
          alreadyGranted={alreadyGranted}
          onToggle={onToggle}
        />
      )}
    </div>
  );
};

// ── Shared checklist ──────────────────────────────────────────────────────────

interface TargetChecklistProps extends TargetsProps {
  label: ReactNode;
  empty: ReactNode;
  isLoading: boolean;
  options: TargetOption[];
}

const TargetChecklist = ({
  label,
  empty,
  isLoading,
  options,
  disabled,
  selected,
  alreadyGranted,
  onToggle,
}: TargetChecklistProps) => (
  <div className='flex flex-col gap-1.5'>
    <Label>{label}</Label>
    {isLoading ? (
      <p className='py-4 text-center text-sm text-muted-foreground'>
        <Trans>Loading…</Trans>
      </p>
    ) : options.length === 0 ? (
      <p className='rounded-lg border border-dashed py-4 text-center text-sm text-muted-foreground'>
        {empty}
      </p>
    ) : (
      <ScrollArea className='max-h-48 rounded-lg border p-2'>
        <div className='flex flex-col gap-0.5'>
          {options.map(option => {
            const granted = alreadyGranted.has(option.id);
            return (
              <label
                key={option.id}
                className='flex items-center gap-2 rounded-md px-2 py-1.5 cursor-pointer hover:bg-secondary aria-disabled:cursor-not-allowed aria-disabled:opacity-60'
                aria-disabled={granted || disabled}
              >
                <Checkbox
                  checked={granted || selected.has(option.id)}
                  disabled={granted || disabled}
                  onCheckedChange={() => onToggle(option)}
                />
                <span className='text-sm truncate'>{option.name}</span>
                {granted && (
                  <Badge variant='outline' className='ml-auto text-[10px]'>
                    <Trans>Granted</Trans>
                  </Badge>
                )}
              </label>
            );
          })}
        </div>
      </ScrollArea>
    )}
  </div>
);

export default GrantCreator;
