import { useEffect, useRef, useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { AlertTriangle, Check, Cpu, Eye, KeyRound } from 'lucide-react';
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

export type SecretEntityKind = 'app' | 'agent';

interface SecretRevealDialogProps {
  open: boolean;
  entityKind: SecretEntityKind;
  entity: { id: string; name: string };
  secret: string;
  /** Placeholder shown in the run command; the operator swaps it in. */
  controlPlaneUrl?: string;
  /** Called once the user confirms they've copied it — caller closes + navigates. */
  onClose: () => void;
}

/**
 * The one-time secret-reveal moment. The secret is shown exactly once and lives
 * only in volatile state — never persisted, logged, or put in a URL. The dialog
 * cannot be dismissed (Escape / outside-click / X) until the user has revealed
 * the secret at least once; the only exit is the confirm button.
 */
export const SecretRevealDialog = ({
  open,
  entityKind,
  entity,
  secret,
  controlPlaneUrl = '<CONTROL_PLANE_URL>',
  onClose,
}: SecretRevealDialogProps) => {
  const [revealed, setRevealed] = useState(false);
  const revealBtnRef = useRef<HTMLButtonElement>(null);
  const confirmBtnRef = useRef<HTMLButtonElement>(null);

  const Icon = entityKind === 'agent' ? Cpu : KeyRound;
  const runCommand = `scylla-agent --control-plane-url ${controlPlaneUrl} \\\n  --app-id ${entity.id} \\\n  --app-secret ${secret}`;

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
        className='[&>button]:hidden w-[calc(100vw-2rem)] sm:max-w-lg p-0 overflow-hidden gap-0'
        onEscapeKeyDown={e => e.preventDefault()}
        onPointerDownOutside={e => e.preventDefault()}
        onInteractOutside={e => e.preventDefault()}
      >
        {/* Amber warning band */}
        <DialogHeader className='space-y-2 border-b border-warning/40 bg-warning/10 p-5'>
          <div className='flex items-center gap-3'>
            <span className='flex h-9 w-9 items-center justify-center rounded-md bg-warning/15 text-warning-foreground'>
              <AlertTriangle className='h-5 w-5 text-warning' />
            </span>
            <DialogTitle className='text-warning-foreground'>
              <Trans>Your secret · shown once</Trans>
            </DialogTitle>
          </div>
          <DialogDescription>
            <Trans>
              We don't store this. Copy it now — closing this dialog deletes it forever.
            </Trans>
          </DialogDescription>
        </DialogHeader>

        <div className='min-w-0 space-y-4 p-5'>
          {/* Confirmation strip: proof the entity was created */}
          <div className='flex items-center gap-3 rounded-md border border-success/30 bg-success/5 p-3'>
            <span className='flex h-8 w-8 items-center justify-center rounded-md border border-success/30 bg-success/10'>
              <Icon className='h-4 w-4 text-success' />
            </span>
            <div className='min-w-0'>
              <p className='truncate text-sm font-semibold'>{entity.name}</p>
              <p className='truncate font-mono text-xs text-muted-foreground'>{entity.id}</p>
            </div>
          </div>

          {/* Secret */}
          <CodeSnippet
            variant='warning'
            multiline
            value={secret}
            blurred={!revealed}
            copyToast='Secret copied'
            label={entityKind === 'agent' ? <Trans>Agent secret</Trans> : <Trans>App secret</Trans>}
            overlay={
              <Button
                ref={revealBtnRef}
                size='sm'
                variant='outline'
                autoFocus
                onClick={() => setRevealed(true)}
                className='gap-2 border-warning/50'
              >
                <Eye className='h-4 w-4' />
                <Trans>click to reveal</Trans>
              </Button>
            }
          />

          {/* Run command — agents only. An App is just a credential; there is
              nothing to "run", so we don't show an agent launch command for it. */}
          {entityKind === 'agent' ? (
            <CodeSnippet
              variant='dark'
              multiline
              value={runCommand}
              blurred={!revealed}
              copyToast='Command copied'
              label={<Trans>Run the agent</Trans>}
            />
          ) : (
            <div className='rounded-md border-l-2 border-success bg-success/5 p-3 text-xs text-muted-foreground'>
              <Trans>
                Use these credentials (id + secret) to authenticate an automation against the Scylla
                API.
              </Trans>
            </div>
          )}
        </div>

        <DialogFooter className='items-center justify-between gap-2 border-t border-dashed border-warning/40 p-4 sm:justify-between'>
          <span className='flex items-center gap-1.5 text-xs text-warning'>
            <AlertTriangle className='h-3.5 w-3.5' />
            <Trans>You won't see this secret again.</Trans>
          </span>
          <Button
            ref={confirmBtnRef}
            disabled={!revealed}
            title={revealed ? undefined : 'Reveal the secret first'}
            onClick={onClose}
          >
            <Check className='mr-1.5 h-4 w-4' />
            <Trans>I've copied it — close</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default SecretRevealDialog;
