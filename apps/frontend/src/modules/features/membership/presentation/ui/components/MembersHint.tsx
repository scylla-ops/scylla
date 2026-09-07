import type { ReactNode } from 'react';
import { Info } from 'lucide-react';

interface MembersHintProps {
  children: ReactNode;
}

/**
 * The footnote under a member list, explaining what the list contains that the
 * rows themselves cannot say — who is listed without holding a role here, and
 * what the reader is not allowed to see.
 */
export const MembersHint = ({ children }: MembersHintProps) => (
  <p className='flex items-start gap-2 text-xs text-muted-foreground'>
    <Info className='mt-0.5 size-3.5 shrink-0' />
    <span>{children}</span>
  </p>
);

export default MembersHint;
