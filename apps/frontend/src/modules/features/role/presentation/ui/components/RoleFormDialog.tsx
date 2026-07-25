import { useEffect, useState } from 'react';
import {
  Badge,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import { Checkbox } from '@shadcn/checkbox.tsx';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { Trans, useLingui } from '@lingui/react/macro';
import { ShieldCheck } from 'lucide-react';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import {
  type Permission,
  PermissionScope,
  type AccessSpec,
} from '@/modules/features/role/domain/structs/permission.struct.ts';
import { useRoles } from '@/modules/features/role/presentation/hooks/use-roles.ts';
import {
  ALL_PERMISSIONS,
  ALL_SCOPES,
  permissionName,
  scopeName,
} from '@/modules/features/role/presentation/utils/authz-labels.ts';
import { PERMISSIONS_BY_SCOPE } from '@/modules/features/role/presentation/utils/permission-mapping.ts';

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
        <DialogHeader className='space-y-3'>
          <DialogTitle className='flex items-center gap-2.5 text-lg font-semibold'>
            <div className='flex items-center justify-center size-8 rounded-lg bg-primary/10'>
              <ShieldCheck className='size-4 text-primary' />
            </div>
            <span>{isEdit ? <Trans>Edit role</Trans> : <Trans>Create role</Trans>}</span>
          </DialogTitle>
          <DialogDescription>
            <Trans>Define what this role is called and what it can do.</Trans>
          </DialogDescription>
        </DialogHeader>

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
              onValueChange={value => setScope(Number(value))}
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
            <div className='flex flex-col gap-1.5'>
              <div className='flex items-center justify-between'>
                <Label>
                  <Trans>Permissions</Trans>
                </Label>
                <Badge variant='secondary'>
                  <Trans>{permissions.size} selected</Trans>
                </Badge>
              </div>
              <ScrollArea className='h-56 rounded-lg border border-slate-200 p-2'>
                <div className='flex flex-col gap-0.5'>
                  {PERMISSIONS_BY_SCOPE.get(scope)?.map(permission => (
                    <label
                      key={permission}
                      className='flex items-center gap-2 rounded-md px-2 py-1.5 cursor-pointer hover:bg-slate-50'
                    >
                      <Checkbox
                        checked={permissions.has(permission)}
                        disabled={isPending}
                        onCheckedChange={() => togglePermission(permission)}
                      />
                      <span className='text-sm capitalize'>{permissionName(permission)}</span>
                    </label>
                  ))}
                </div>
              </ScrollArea>
            </div>
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
