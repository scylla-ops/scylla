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
import { Globe, Plus, X } from 'lucide-react';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import {
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/role/domain/structs/permission.struct.ts';
import { useGrants } from '@/modules/features/role/presentation/hooks/use-grants.ts';
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

/**
 * Grants a role to a user within the scope the role requires:
 * - SYSTEM       → the user, system-wide (no scope id).
 * - ORGANIZATION → the user, on one or more organizations.
 * - PROJECT      → the user, on one or more projects of a chosen organization.
 *
 * One grant is created per selected scope target.
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

  const userItems = users?.items ?? [];

  // Scope ids where this user already holds this role — offered as disabled.
  const alreadyGranted = useMemo(() => {
    const ids = new Set<string>();
    if (!userId) return ids;
    for (const grant of grants) {
      if (
        grant.target.kind === 'role' &&
        grant.target.roleId === role.id &&
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
            target: { kind: 'role', roleId: role.id },
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

  return (
    <>
      <Button size='sm' onClick={() => setOpen(true)}>
        <Plus className='size-4' />
        <Trans>Add grant</Trans>
      </Button>

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
            {/* User picker */}
            <div className='flex flex-col gap-1.5'>
              <Label htmlFor='grant-user'>
                <Trans>User</Trans>
              </Label>
              <Select value={userId} disabled={isPending} onValueChange={setUserId}>
                <SelectTrigger id='grant-user' className='w-full'>
                  <SelectValue placeholder={t`Select a user`} />
                </SelectTrigger>
                <SelectContent>
                  {userItems.map(user => (
                    <SelectItem key={user.userId} value={user.userId}>
                      {user.username}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {role.scope === PermissionScope.SYSTEM && (
              <div className='flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm text-muted-foreground'>
                <Globe className='size-4 shrink-0' />
                <Trans>This role grants access across the whole system.</Trans>
              </div>
            )}

            {role.scope === PermissionScope.ORGANIZATION && (
              <OrganizationTargets
                disabled={isPending}
                selected={selected}
                alreadyGranted={alreadyGranted}
                onToggle={toggle}
              />
            )}

            {role.scope === PermissionScope.PROJECT && (
              <ProjectTargets
                disabled={isPending}
                browseOrgId={browseOrgId}
                onBrowseOrg={setBrowseOrgId}
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

interface ProjectTargetsProps extends TargetsProps {
  browseOrgId: string | null;
  onBrowseOrg: (orgId: string) => void;
}

const ProjectTargets = ({
  disabled,
  browseOrgId,
  onBrowseOrg,
  selected,
  alreadyGranted,
  onToggle,
}: ProjectTargetsProps) => {
  const { organizations, isLoading: orgsLoading } = useOrganizations();
  const { projects, isLoading: projectsLoading } = useProjects(browseOrgId);

  const options: TargetOption[] = (projects ?? []).map(project => ({
    id: project.id,
    name: project.name,
  }));

  return (
    <div className='flex flex-col gap-3'>
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

      {browseOrgId && (
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
      <p className='rounded-lg border border-dashed border-slate-200 py-4 text-center text-sm text-muted-foreground'>
        {empty}
      </p>
    ) : (
      <ScrollArea className='max-h-48 rounded-lg border border-slate-200 p-2'>
        <div className='flex flex-col gap-0.5'>
          {options.map(option => {
            const granted = alreadyGranted.has(option.id);
            return (
              <label
                key={option.id}
                className='flex items-center gap-2 rounded-md px-2 py-1.5 cursor-pointer hover:bg-slate-50 aria-disabled:cursor-not-allowed aria-disabled:opacity-60'
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
