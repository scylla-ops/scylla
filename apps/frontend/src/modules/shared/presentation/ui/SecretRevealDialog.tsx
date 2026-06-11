import { useEffect, useRef, useState } from 'react';
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
import { AgentRunInstructions } from './AgentRunInstructions.tsx';

export type SecretEntityKind = 'app' | 'agent';

interface SecretRevealDialogProps {
  open: boolean;
  entityKind: SecretEntityKind;
  entity: { id: string; name: string };
  secret: string;
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
  entityKind,
  entity,
  secret,
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

  const hasRunStep = entityKind === 'agent';

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
          <DialogTitle className='text-[15px]'>
            {hasRunStep ? (
              <Trans>{entity.name} is ready</Trans>
            ) : (
              <Trans>{entity.name} credentials</Trans>
            )}
          </DialogTitle>
          <DialogDescription className='text-[13px]'>
            {hasRunStep ? (
              <Trans>Two steps and it picks up jobs.</Trans>
            ) : (
              <Trans>Copy the secret below — it is shown only once.</Trans>
            )}
          </DialogDescription>
        </DialogHeader>

        <div className='space-y-1 px-5 pb-4'>
          {/* Step 1 — the secret */}
          <div className='flex gap-3'>
            <div className='flex flex-col items-center'>
              <StepBullet n={1} active />
              {hasRunStep && <span className='mt-1.5 w-px flex-1 bg-border' />}
            </div>
            <div className='min-w-0 flex-1 pb-4'>
              <p className='mb-2 mt-0.5 text-[13px] font-medium'>
                <Trans>Copy your secret</Trans>{' '}
                <span className='font-normal text-muted-foreground'>
                  · <Trans>shown once</Trans>
                </span>
              </p>
              <CodeSnippet
                multiline
                value={secret}
                blurred={!revealed}
                copyToast='Secret copied'
                label={
                  <span className='truncate font-mono'>
                    {entity.id}
                  </span>
                }
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

          {/* Step 2 — run it (agents); apps just get a quiet usage note */}
          {hasRunStep ? (
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
                <p className='mb-2 mt-0.5 text-[13px] font-medium'>
                  <Trans>Start your worker</Trans>
                </p>
                {revealed && <AgentRunInstructions appId={entity.id} secret={secret} />}
              </div>
            </div>
          ) : (
            revealed && (
              <p className='pl-9 text-xs text-muted-foreground'>
                <Trans>
                  Use these credentials (id + secret) to authenticate an automation against the
                  Scylla API.
                </Trans>
              </p>
            )
          )}
        </div>

        <DialogFooter className='items-center justify-between gap-2 border-t p-4 sm:justify-between'>
          <span className='text-xs text-muted-foreground'>
            <Trans>You won't see this secret again.</Trans>
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
