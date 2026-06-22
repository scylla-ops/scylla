import { useState, type ReactNode } from 'react';
import { Check, Copy } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { Button } from '@shadcn';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { cn } from '@shared/presentation/utils';

interface CopyableTextProps {
  /** The full text written to the clipboard. */
  value: string;
  /** Truncate the displayed text to N characters (followed by an ellipsis). */
  truncate?: number;
  /** Override what is rendered; defaults to `value` (truncated when `truncate` is set). */
  display?: ReactNode;
  /** Show the full `value` in a tooltip when hovering the text. */
  showFullOnHover?: boolean;
  /** Tooltip label on the copy button (default: "Copy"). */
  copyLabel?: ReactNode;
  /** Extra classes for the root wrapper. */
  className?: string;
}

/**
 * Inline monospace text with a copy-to-clipboard button. The button toggles to a
 * checkmark for 2s after copying. Used in table cells (job ids, webhook urls) so the
 * copy affordance is identical everywhere instead of hand-rolled per cell.
 *
 * For a boxed credential/code block with a header bar, use `CodeSnippet` instead.
 */
export const CopyableText = ({
  value,
  truncate,
  display,
  showFullOnHover = false,
  copyLabel,
  className,
}: CopyableTextProps) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const text = display ?? (truncate ? `${value.slice(0, truncate)}...` : value);
  const label = <span className='font-mono text-sm truncate'>{text}</span>;

  return (
    <div className={cn('flex items-center gap-2', className)}>
      {showFullOnHover ? (
        <Tooltip>
          <TooltipTrigger asChild>{label}</TooltipTrigger>
          <TooltipContent>
            <p>{value}</p>
          </TooltipContent>
        </Tooltip>
      ) : (
        label
      )}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            size='icon'
            variant='ghost'
            className='h-6 w-6 shrink-0 cursor-pointer transition-all hover:scale-125 hover:text-primary hover:bg-primary-hover rounded-full'
            onClick={handleCopy}
          >
            {copied ? <Check className='w-3 h-3 text-green-500' /> : <Copy className='w-3 h-3' />}
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>{copied ? <Trans>Copied!</Trans> : (copyLabel ?? <Trans>Copy</Trans>)}</p>
        </TooltipContent>
      </Tooltip>
    </div>
  );
};

export default CopyableText;
