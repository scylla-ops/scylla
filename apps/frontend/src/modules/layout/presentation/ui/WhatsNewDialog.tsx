import { Trans, useLingui } from '@lingui/react/macro';
import { Button } from '@shadcn';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@shadcn/dialog.tsx';
import { useUnseenRelease } from '@/modules/layout/presentation/hooks/use-whats-new.ts';

/**
 * Announces the current release once, from the `whats-new.ts` declaration.
 * Nothing to write here when a feature ships — only a highlight to add there.
 */
export const WhatsNewDialog = () => {
  const { release, dismiss } = useUnseenRelease();
  const { i18n } = useLingui();

  if (!release) return null;

  return (
    <Dialog open onOpenChange={open => !open && dismiss()}>
      <DialogContent className='sm:max-w-lg'>
        <DialogHeader>
          <DialogTitle>
            <Trans>What's new in Scylla {release.version}</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Here is what landed in this version.</Trans>
          </DialogDescription>
        </DialogHeader>

        <ul className='flex flex-col gap-4 py-2'>
          {release.highlights.map(highlight => (
            <li key={highlight.id} className='flex items-start gap-3'>
              <span className='flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10'>
                <highlight.icon className='size-4 text-primary' />
              </span>
              <div className='min-w-0'>
                <p className='text-sm font-semibold text-foreground'>{i18n._(highlight.title)}</p>
                <p className='text-sm text-muted-foreground'>{i18n._(highlight.description)}</p>
              </div>
            </li>
          ))}
        </ul>

        <DialogFooter>
          <Button onClick={dismiss}>
            <Trans>Got it</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
