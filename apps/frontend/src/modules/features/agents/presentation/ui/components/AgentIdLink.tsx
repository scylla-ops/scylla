import { Copy } from 'lucide-react';
import { toast } from 'sonner';
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
 * The agent id, click-to-copy: one click puts the full id in the clipboard
 * and confirms with a toast. Shown truncated in tight spots (cards), full in
 * the detail strip — the copy always carries the complete id.
 */
export const AgentIdLink = ({ id, truncate, chip = false, className }: AgentIdLinkProps) => {
  const label = truncate && id.length > truncate ? `${id.slice(0, truncate)}…` : id;

  const copyId = async (e: React.MouseEvent) => {
    // Cards navigate on click — copying must not also open the agent.
    e.stopPropagation();
    try {
      await navigator.clipboard.writeText(id);
      toast.success('Agent id copied');
    } catch {
      // Clipboard can be denied (permissions, non-secure context) — say so
      // instead of failing silently.
      toast.error('Could not copy — select the id manually');
    }
  };

  return (
    <button
      type='button'
      onClick={e => void copyId(e)}
      title='Copy agent id'
      aria-label='Copy agent id'
      className={cn(
        'group inline-flex cursor-pointer items-center gap-1 font-mono text-xs text-foreground transition-colors duration-100',
        'hover:text-success hover:underline hover:decoration-success hover:underline-offset-[3px]',
        chip &&
          'rounded border border-border bg-muted/60 px-1.5 py-0.5 hover:border-success/60 hover:bg-success/10',
        className,
      )}
    >
      <span className='truncate'>{label}</span>
      <Copy className='h-2.5 w-2.5 shrink-0 opacity-55 transition-colors group-hover:text-success group-hover:opacity-100' />
    </button>
  );
};
