import { Link, useParams } from 'react-router-dom';
import { ArrowUpRight } from 'lucide-react';
import { cn } from '@shared/presentation/utils';

interface AgentIdLinkProps {
  id: string;
  /** Truncate to N chars with an ellipsis (full id when omitted). */
  truncate?: number;
  /** Chip variant: paperWarm background + faint border at rest. */
  chip?: boolean;
  className?: string;
}

/**
 * The agent id is always a link to the credential view of the same id
 * (the underlying App). Same id, two views of one entity. Clicking copies
 * nothing — copying lives in the agent card ellipsis menu / App detail.
 */
export const AgentIdLink = ({ id, truncate, chip = false, className }: AgentIdLinkProps) => {
  const { organizationSlug } = useParams<{ organizationSlug: string }>();
  const label = truncate && id.length > truncate ? `${id.slice(0, truncate)}…` : id;

  return (
    <Link
      to={`/${organizationSlug}/apps/${id}`}
      onClick={e => e.stopPropagation()}
      title={id}
      className={cn(
        'group inline-flex items-center gap-1 font-mono text-xs text-foreground transition-colors duration-100',
        'hover:text-success hover:underline hover:decoration-success hover:underline-offset-[3px]',
        chip &&
          'rounded border border-border bg-muted/60 px-1.5 py-0.5 hover:border-success/60 hover:bg-success/10',
        className,
      )}
    >
      <span className='truncate'>{label}</span>
      <ArrowUpRight className='h-2.5 w-2.5 shrink-0 opacity-55 transition-colors group-hover:text-success group-hover:opacity-100' />
    </Link>
  );
};
