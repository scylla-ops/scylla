import type { ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { Label } from '@shadcn';
import { Checkbox } from '@shadcn/checkbox.tsx';
import type { AssignableRole } from '@/modules/features/permission/presentation/hooks/use-assignable-roles.ts';

interface RoleChecklistProps {
  label: ReactNode;
  roles: AssignableRole[];
  isLoading?: boolean;
  disabled?: boolean;
  /** Role ids currently ticked. */
  selected: Set<string>;
  /** Role ids the member already holds — ticked, locked, labelled as such. */
  alreadyHeld?: Set<string>;
  onToggle: (roleId: string) => void;
}

/**
 * Pick any number of roles at once.
 *
 * Multi-select rather than a single picker because a member's access is the sum
 * of their roles, not one of them: granting "developer" and "secrets reader"
 * together is the normal case, and forcing it through one grant at a time
 * turned a single decision into a sequence of them.
 *
 * The list scrolls in a plain overflow container rather than the shared
 * `ScrollArea`: that component's root is only `relative`, so a `max-h-*` on it
 * clips nothing and a long role list spills over whatever follows.
 */
export const RoleChecklist = ({
  label,
  roles,
  isLoading = false,
  disabled = false,
  selected,
  alreadyHeld,
  onToggle,
}: RoleChecklistProps) => (
  <div className='flex min-w-0 flex-col gap-1.5'>
    <Label>{label}</Label>
    {isLoading ? (
      <p className='py-4 text-center text-sm text-muted-foreground'>
        <Trans>Loading…</Trans>
      </p>
    ) : roles.length === 0 ? (
      <p className='rounded-lg border border-dashed border-border py-4 text-center text-sm text-muted-foreground'>
        <Trans>No role can be granted here.</Trans>
      </p>
    ) : (
      <div className='max-h-48 overflow-y-auto rounded-lg border border-border p-1'>
        <div className='flex flex-col'>
          {roles.map(role => {
            const held = alreadyHeld?.has(role.roleId) ?? false;
            return (
              <label
                key={role.roleId}
                className='flex cursor-pointer items-start gap-2 rounded-md px-2 py-1.5 hover:bg-accent/50 aria-disabled:cursor-not-allowed aria-disabled:opacity-60'
                aria-disabled={held || disabled}
              >
                <Checkbox
                  className='mt-0.5 shrink-0'
                  checked={held || selected.has(role.roleId)}
                  disabled={held || disabled}
                  onCheckedChange={() => onToggle(role.roleId)}
                />
                <span className='flex min-w-0 flex-1 flex-col gap-0.5'>
                  <span className='truncate text-sm leading-tight'>{role.name}</span>
                  {role.description && (
                    <span className='text-xs leading-snug text-muted-foreground'>
                      {role.description}
                    </span>
                  )}
                </span>
                {held && (
                  <span className='shrink-0 self-center text-[10px] uppercase tracking-wide text-muted-foreground'>
                    <Trans>Held</Trans>
                  </span>
                )}
              </label>
            );
          })}
        </div>
      </div>
    )}
  </div>
);

export default RoleChecklist;
