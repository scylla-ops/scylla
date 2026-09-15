import { useState, type ReactNode, type SyntheticEvent } from 'react';
import { Check, Copy } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { cn } from '@shared/presentation/utils';
// Direct path, not the `ui` barrel: importing it from here would loop back through data-display.
import { IconButton } from '../controls/IconButton.tsx';

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

  const handleCopy = (e: SyntheticEvent) => {
    e.stopPropagation();
    void navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const text = display ?? (truncate ? `${value.slice(0, truncate)}...` : value);
  const label = <span className='font-mono truncate'>{text}</span>;

  // `min-w-0` on the root so the label can actually ellipsize when this sits in a flex row.
  return (
    <div className={cn('flex min-w-0 items-center gap-2', className)}>
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
      <IconButton
        icon={copied ? Check : Copy}
        tooltip={copied ? <Trans>Copied!</Trans> : (copyLabel ?? <Trans>Copy</Trans>)}
        onClick={handleCopy}
        className='shrink-0'
        iconClassName={copied ? 'text-status-passed' : undefined}
      />
    </div>
  );
};

export default CopyableText;
