import { useEffect, useRef, useState, type ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { Check, Eye } from 'lucide-react';
import { Button } from '@shadcn';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@shadcn/dialog.tsx';
import { CodeSnippet } from '@shadcn/code-snippet.tsx';
import { cn } from '@shared/presentation/utils';

interface SecretRevealDialogProps {
  open: boolean;
  title: ReactNode;
  description: ReactNode;
  /** The one-time secret value shown (blurred until revealed). */
  secret: string;
  /** Label displayed on the secret snippet (e.g. the entity id). */
  secretLabel: ReactNode;
  /** Toast shown after copying; defaults to "Secret copied". */
  copyToast?: string;
  /** Heading for the secret step; defaults to "Copy your secret". */
  secretStepTitle?: ReactNode;
  /**
   * Optional numbered second step, revealed once the secret is shown — e.g. run
   * instructions. When present, the checklist connector + step 2 bullet appear.
   */
  secondStep?: { title: ReactNode; content: ReactNode };
  /** Optional quiet note shown under the secret once revealed (no second step). */
  revealedNote?: ReactNode;
  /** Footer note; defaults to "You won't see this secret again." */
  footerNote?: ReactNode;
  /** Called once the user confirms they've copied it — caller closes + navigates. */
  onClose: () => void;
}

/** Numbered step bullet: filled with the accent once the step is reachable. */
const StepBullet = ({ n, active }: { n: number; active: boolean }) => (
  <span
    className={cn(
      'flex h-[22px] w-[22px] shrink-0 items-center justify-center rounded-full text-[11px] font-medium transition-colors',
      active
        ? 'bg-primary text-primary-foreground'
        : 'border-[1.5px] border-border text-muted-foreground',
    )}
  >
    {n}
  </span>
);

/**
 * The one-time secret-reveal moment, as a quiet two-step checklist: copy the
 * secret, then start the worker. One accent color (primary), no warning
 * banners — the "shown once" stake is carried by the copy, not by paint. The
 * dialog cannot be dismissed until the secret has been revealed at least once.
 */
export const SecretRevealDialog = ({
  open,
  title,
  description,
  secret,
  secretLabel,
  copyToast = 'Secret copied',
  secretStepTitle,
  secondStep,
  revealedNote,
  footerNote,
  onClose,
}: SecretRevealDialogProps) => {
  const [revealed, setRevealed] = useState(false);
  const revealBtnRef = useRef<HTMLButtonElement>(null);
  const confirmBtnRef = useRef<HTMLButtonElement>(null);

  // Reset to the hidden phase whenever the dialog (re)opens.
  useEffect(() => {
    if (open) setRevealed(false);
  }, [open]);

  // Move focus to the confirm button once the secret is revealed.
  useEffect(() => {
    if (revealed) confirmBtnRef.current?.focus();
  }, [revealed]);

  return (
    <Dialog open={open}>
      <DialogContent
        // No close affordances until revealed: hide X, block escape + outside click.
        className='[&>button]:hidden w-[calc(100vw-2rem)] sm:max-w-lg gap-0 p-0'
        onEscapeKeyDown={e => e.preventDefault()}
        onPointerDownOutside={e => e.preventDefault()}
        onInteractOutside={e => e.preventDefault()}
      >
        <DialogHeader className='space-y-1 p-5 pb-3 text-left'>
          <DialogTitle className='text-[15px]'>{title}</DialogTitle>
          <DialogDescription className='text-[13px]'>{description}</DialogDescription>
        </DialogHeader>

        <div className='space-y-1 px-5 pb-4'>
          {/* Step 1 — the secret */}
          <div className='flex gap-3'>
            <div className='flex flex-col items-center'>
              <StepBullet n={1} active />
              {secondStep && <span className='mt-1.5 w-px flex-1 bg-border' />}
            </div>
            <div className='min-w-0 flex-1 pb-4'>
              <p className='mb-2 mt-0.5 text-[13px] font-medium'>
                {secretStepTitle ?? <Trans>Copy your secret</Trans>}{' '}
                <span className='font-normal text-muted-foreground'>
                  · <Trans>shown once</Trans>
                </span>
              </p>
              <CodeSnippet
                multiline
                value={secret}
                blurred={!revealed}
                copyToast={copyToast}
                label={<span className='truncate font-mono'>{secretLabel}</span>}
                overlay={
                  <Button
                    ref={revealBtnRef}
                    size='sm'
                    variant='outline'
                    autoFocus
                    onClick={() => setRevealed(true)}
                    className='gap-2'
                  >
                    <Eye className='h-4 w-4' />
                    <Trans>Reveal</Trans>
                  </Button>
                }
              />
            </div>
          </div>

          {/* Step 2 — an optional follow-up (e.g. run instructions); otherwise a quiet note */}
          {secondStep ? (
            <div className='flex gap-3'>
              <div className='flex flex-col items-center'>
                <StepBullet n={2} active={revealed} />
              </div>
              <div
                className={cn(
                  'min-w-0 flex-1 transition-opacity',
                  revealed ? 'opacity-100' : 'opacity-40',
                )}
              >
                <p className='mb-2 mt-0.5 text-[13px] font-medium'>{secondStep.title}</p>
                {revealed && secondStep.content}
              </div>
            </div>
          ) : (
            revealed &&
            revealedNote && <p className='pl-9 text-xs text-muted-foreground'>{revealedNote}</p>
          )}
        </div>

        <DialogFooter className='items-center justify-between gap-2 border-t p-4 sm:justify-between'>
          <span className='text-xs text-muted-foreground'>
            {footerNote ?? <Trans>You won't see this secret again.</Trans>}
          </span>
          <Button
            ref={confirmBtnRef}
            disabled={!revealed}
            title={revealed ? undefined : 'Reveal the secret first'}
            onClick={onClose}
          >
            <Check className='mr-1.5 h-4 w-4' />
            <Trans>Done</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default SecretRevealDialog;
