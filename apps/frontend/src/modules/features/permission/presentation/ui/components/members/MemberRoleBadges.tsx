import type { ReactNode } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Lock, X } from 'lucide-react';
import { cn } from '@shared/presentation/utils';
import {
  MemberRoleOrigin,
  type MemberRole,
} from '@/modules/features/permission/domain/structs/scope-member.struct.ts';
import { usePermissionLabels } from '@/modules/features/permission/presentation/hooks/use-permission-labels.ts';

interface MemberRoleBadgesProps {
  roles: MemberRole[];
  /** Display name for a role id — see `useAssignableRoles().labelFor`. */
  labelFor: (roleId: string) => string;
  /** Absent, or false, makes every chip read-only (no revoke affordance). */
  canManage?: boolean;
  disabled?: boolean;
  onRevoke?: (role: MemberRole) => void;
  /** Shown when the member holds no role at all. */
  empty?: ReactNode;
}

/**
 * The chip is hand-rolled rather than built on `Badge`: `Badge` is a
 * fixed-height, `overflow-hidden` pill meant to hold a word, and the two things
 * these chips must carry inside them — a scope pastille and a revoke control —
 * get clipped by it. Same visual language, one row height, nothing spilling.
 */
const CHIP =
  'inline-flex h-6 max-w-full items-center gap-1.5 rounded-full border py-0.5 pl-2.5 text-xs leading-none';

/** Held here, and editable here. */
const DIRECT_CHIP = 'border-transparent bg-secondary font-medium text-secondary-foreground';

/** Comes from above and is administered there: quieter, dashed, locked. */
const INHERITED_CHIP = 'border-dashed border-border bg-muted/40 text-muted-foreground';

/**
 * A member's roles, one chip each.
 *
 * The distinction the component exists for is direct vs inherited: from a
 * project, a role granted on the project and a role inherited from the
 * organization look identical in their effect and are completely different to
 * administer. Only the first can be revoked here — the second is shown locked,
 * with the reason, rather than hidden, because hiding it would make the project
 * look like it grants less access than it does.
 *
 * The scope pastille rides along on the inherited chips **only**, where it is
 * the answer to "then where do I change it?". On a direct chip it would repeat
 * the scope of the page on every chip of every card — noise that made the chips
 * twice as wide and pushed the rest of the roles out of sight.
 */
export const MemberRoleBadges = ({
  roles,
  labelFor,
  canManage = false,
  disabled = false,
  onRevoke,
  empty,
}: MemberRoleBadgesProps) => {
  const { t } = useLingui();
  const { scopeLabel } = usePermissionLabels();

  if (roles.length === 0) {
    return (
      <span className='text-xs italic text-muted-foreground'>
        {empty ?? <Trans>No role</Trans>}
      </span>
    );
  }

  return (
    <div className='flex min-w-0 flex-wrap items-center gap-1.5'>
      {roles.map(role => {
        const inherited = role.origin === MemberRoleOrigin.INHERITED;
        const name = labelFor(role.roleId);

        const trailing = inherited ? (
          <Lock className='size-3 shrink-0' />
        ) : canManage ? (
          <button
            type='button'
            disabled={disabled}
            aria-label={t`Revoke ${name}`}
            className='inline-flex size-4 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-destructive/10 hover:text-destructive disabled:pointer-events-none disabled:opacity-50'
            onClick={() => onRevoke?.(role)}
          >
            <X className='size-3' />
          </button>
        ) : null;

        const chip = (
          <span
            className={cn(
              CHIP,
              // A chip with nothing trailing keeps its symmetric padding; one
              // carrying a control tucks it into the gap instead.
              trailing ? 'pr-1.5' : 'pr-2.5',
              inherited ? INHERITED_CHIP : DIRECT_CHIP,
            )}
          >
            <span className='max-w-[10rem] truncate'>{name}</span>
            {inherited && (
              <span className='shrink-0 rounded-full bg-background/70 px-1.5 py-0.5 text-[10px] uppercase leading-none tracking-wide'>
                {scopeLabel(role.scope)}
              </span>
            )}
            {trailing}
          </span>
        );

        if (!inherited) return <span key={role.grantId}>{chip}</span>;

        return (
          <Tooltip key={role.grantId}>
            <TooltipTrigger asChild>
              <span className='inline-flex max-w-full'>{chip}</span>
            </TooltipTrigger>
            <TooltipContent>
              <Trans>Managed at the organization level.</Trans>
            </TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
};

export default MemberRoleBadges;
