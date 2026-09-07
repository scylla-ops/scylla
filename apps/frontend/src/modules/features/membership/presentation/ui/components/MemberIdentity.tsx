import type { ReactNode } from 'react';

interface MemberIdentityProps {
  name: string;
  /** The line under the name — on a card, how many roles they hold. */
  subtitle?: ReactNode;
}

/**
 * A member's name, with the initial-avatar that makes a long list scannable.
 *
 * The avatar is derived rather than fetched: the backend's member lists carry a
 * username and nothing else, and a row of identical placeholder pictures would
 * be noise. One letter per person is enough to give the eye an anchor.
 */
export const MemberIdentity = ({ name, subtitle }: MemberIdentityProps) => (
  <span className='flex min-w-0 items-center gap-3'>
    <span
      aria-hidden
      className='flex size-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold uppercase text-primary'
    >
      {name.charAt(0)}
    </span>
    <span className='flex min-w-0 flex-col gap-0.5'>
      <span className='truncate text-sm font-semibold leading-tight' title={name}>
        {name}
      </span>
      {subtitle && (
        <span className='truncate text-xs leading-tight text-muted-foreground'>{subtitle}</span>
      )}
    </span>
  </span>
);

export default MemberIdentity;
