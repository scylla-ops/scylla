import * as React from 'react';
import { Check, Copy } from 'lucide-react';
import { toast } from 'sonner';
import { cn } from '@shared/presentation/utils';

type CodeSnippetVariant = 'default' | 'dark' | 'warning';

const VARIANT: Record<
  CodeSnippetVariant,
  { box: string; header: string; body: string; copy: string }
> = {
  default: {
    box: 'border bg-muted/50',
    header: 'border-border/60 text-muted-foreground',
    body: 'text-foreground',
    copy: 'text-muted-foreground hover:bg-foreground/10',
  },
  dark: {
    box: 'border-transparent bg-foreground',
    header: 'border-background/15 text-background/60',
    body: 'text-background',
    copy: 'text-background/70 hover:bg-background/15',
  },
  warning: {
    box: 'border-warning/40 bg-warning/10',
    header: 'border-warning/30 text-warning',
    body: 'text-foreground',
    copy: 'text-warning hover:bg-warning/20',
  },
};

interface CodeSnippetProps {
  /** Text shown and copied. */
  value: string;
  /** Header label on the left of the top bar. */
  label?: React.ReactNode;
  variant?: CodeSnippetVariant;
  /** Wrap long lines instead of letting them overflow. */
  multiline?: boolean;
  /** Blur the body + disable copy (one-time-secret pattern). */
  blurred?: boolean;
  /** Centered overlay over the body (e.g. a "reveal" button) while blurred. */
  overlay?: React.ReactNode;
  /** Toast message on successful copy. */
  copyToast?: string;
  className?: string;
}

/**
 * A code/credential box: a thin header bar carrying an optional label and a
 * copy button (top-right), with a monospace body beneath. One component so the
 * copy affordance is consistent everywhere instead of hand-rolled per call.
 */
export const CodeSnippet = ({
  value,
  label,
  variant = 'default',
  multiline = false,
  blurred = false,
  overlay,
  copyToast = 'Copied',
  className,
}: CodeSnippetProps) => {
  const [copied, setCopied] = React.useState(false);
  const styles = VARIANT[variant];

  const copy = async () => {
    if (blurred) return;
    await navigator.clipboard.writeText(value);
    setCopied(true);
    toast.success(copyToast);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className={cn('overflow-hidden rounded-md border', styles.box, className)}>
      <div
        className={cn(
          'flex items-center justify-between gap-2 border-b px-3 py-1.5',
          styles.header,
        )}
      >
        <span className='font-mono text-[10px] uppercase tracking-wide'>{label}</span>
        <button
          type='button'
          aria-label='Copy'
          onClick={copy}
          disabled={blurred}
          className={cn(
            'rounded p-1 transition-colors disabled:cursor-not-allowed disabled:opacity-40',
            styles.copy,
          )}
        >
          {copied ? <Check className='h-3.5 w-3.5' /> : <Copy className='h-3.5 w-3.5' />}
        </button>
      </div>
      <div className='relative'>
        <pre
          className={cn(
            'px-3 py-2.5 font-mono text-xs',
            multiline ? 'whitespace-pre-wrap break-all' : 'overflow-x-auto',
            styles.body,
            blurred && 'secret-blur',
          )}
        >
          {value}
        </pre>
        {blurred && overlay && (
          <div className='absolute inset-0 flex items-center justify-center'>{overlay}</div>
        )}
      </div>
    </div>
  );
};

export default CodeSnippet;
