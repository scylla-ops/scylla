import { useEffect, useState } from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import { Trans, useLingui } from '@lingui/react/macro';
import type { RoleEntity } from '@/modules/features/permission/domain/entities/role.entity.ts';
import {
  type Permission,
  PermissionScope,
  type AccessSpec,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useRoles } from '@/modules/features/permission/presentation/hooks/use-roles.ts';
import { RoleDialogHeader } from '@/modules/features/permission/presentation/ui/components/role-form/RoleDialogHeader.tsx';
import { RoleDialogPermissions } from '@/modules/features/permission/presentation/ui/components/role-form/RoleDialogPermissions.tsx';
import {
  ALL_SCOPES,
  getPermissionsForScope,
  scopeName,
} from '@/modules/features/permission/presentation/utils/permission-mapping.ts';

type AccessKind = 'fullControl' | 'restricted';

interface RoleFormDialogProps {
  open: boolean;
  /** The role being edited, or `null` when creating a new one. */
  role: RoleEntity | null;
  onClose: () => void;
}

export const RoleFormDialog = ({ open, role, onClose }: RoleFormDialogProps) => {
  const { t } = useLingui();
  const { createRole, updateRole } = useRoles();
  const isEdit = role !== null;

  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [scope, setScope] = useState<PermissionScope>(PermissionScope.ORGANIZATION);
  const [accessKind, setAccessKind] = useState<AccessKind>('restricted');
  const [permissions, setPermissions] = useState<Set<Permission>>(new Set());

  // Re-seed the form whenever the dialog opens for a different role (or for create).
  useEffect(() => {
    if (!open) return;
    setName(role?.name ?? '');
    setDescription(role?.description ?? '');
    setScope(role?.scope ?? PermissionScope.ORGANIZATION);
    if (role?.access.kind === 'fullControl') {
      setAccessKind('fullControl');
      setPermissions(new Set());
    } else if (role?.access.kind === 'restricted') {
      setAccessKind('restricted');
      setPermissions(new Set(role.access.permissions));
    } else {
      setAccessKind('restricted');
      setPermissions(new Set());
    }
  }, [open, role]);

  const togglePermission = (permission: Permission) =>
    setPermissions(prev => {
      const next = new Set(prev);
      if (next.has(permission)) next.delete(permission);
      else next.add(permission);
      return next;
    });

  const isValid = name.trim().length > 0 && (accessKind === 'fullControl' || permissions.size > 0);
  const isPending = createRole.isPending || updateRole.isPending;

  const buildAccess = (): AccessSpec =>
    accessKind === 'fullControl'
      ? { kind: 'fullControl' }
      : { kind: 'restricted', permissions: [...permissions] };

  const handleSubmit = () => {
    if (!isValid) return;
    if (isEdit) {
      updateRole.mutate(
        { id: role.id, name: name.trim(), description: description.trim(), access: buildAccess() },
        { onSuccess: onClose },
      );
    } else {
      createRole.mutate(
        { name: name.trim(), description: description.trim(), scope, access: buildAccess() },
        { onSuccess: onClose },
      );
    }
  };

  return (
    <Dialog open={open} onOpenChange={value => !value && onClose()}>
      <DialogContent className='max-w-xl flex flex-col max-h-[85vh]'>
        <RoleDialogHeader isEdit={isEdit} />

        <div className='flex flex-col gap-4 overflow-y-auto pr-1'>
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='role-name'>
              <Trans>Name</Trans>
            </Label>
            <Input
              id='role-name'
              autoFocus
              value={name}
              disabled={isPending}
              placeholder={t`e.g., project-viewer`}
              onChange={e => setName(e.target.value)}
            />
          </div>

          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='role-description'>
              <Trans>Description</Trans>
            </Label>
            <Input
              id='role-description'
              value={description}
              disabled={isPending}
              placeholder={t`What is this role for?`}
              onChange={e => setDescription(e.target.value)}
            />
          </div>

          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='role-scope'>
              <Trans>Scope</Trans>
            </Label>
            <Select
              value={String(scope)}
              disabled={isEdit || isPending}
              onValueChange={value => {
                const next: PermissionScope = Number(value);
                setScope(next);
                // Drop any selected permissions that aren't coherent at the new scope.
                const allowed = new Set(getPermissionsForScope(next) ?? []);
                setPermissions(prev => new Set([...prev].filter(p => allowed.has(p))));
              }}
            >
              <SelectTrigger id='role-scope' className='w-full'>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {ALL_SCOPES.map(s => (
                  <SelectItem key={s} value={String(s)}>
                    {scopeName(s)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {isEdit && (
              <p className='text-xs text-muted-foreground'>
                <Trans>Scope cannot be changed after creation.</Trans>
              </p>
            )}
          </div>

          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='role-access'>
              <Trans>Access</Trans>
            </Label>
            <Select
              value={accessKind}
              disabled={isPending}
              onValueChange={value => setAccessKind(value as AccessKind)}
            >
              <SelectTrigger id='role-access' className='w-full'>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='fullControl'>
                  <Trans>Full control</Trans>
                </SelectItem>
                <SelectItem value='restricted'>
                  <Trans>Restricted permissions</Trans>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          {accessKind === 'restricted' && (
            <RoleDialogPermissions
              scope={scope}
              permissions={permissions}
              isPending={isPending}
              togglePermission={togglePermission}
            />
          )}
        </div>

        <DialogFooter>
          <Button type='button' variant='outline' disabled={isPending} onClick={onClose}>
            <Trans>Cancel</Trans>
          </Button>
          <Button type='button' disabled={!isValid || isPending} onClick={handleSubmit}>
            {isEdit ? <Trans>Save changes</Trans> : <Trans>Create role</Trans>}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
