import { useEffect } from 'react';
import { Zap } from 'lucide-react';
import { toast } from 'sonner';
import { useNewFeature } from '@shared/presentation/hooks/use-new-feature.ts';
import { Trans } from '@lingui/react/macro';

const TOAST_ID = 'new-trigger-feature';

export const NewTriggerFeaturePopup = () => {
  const { isNew, markSeen } = useNewFeature('triggers');

  useEffect(() => {
    if (!isNew) return;

    const handleSeen = () => {
      toast.dismiss(TOAST_ID);
      markSeen();
    };

    toast.custom(
      () => (
        <div className='flex w-full items-start gap-3 rounded-xl border border-border bg-background px-4 py-3 shadow-lg'>
          <Zap className='mt-0.5 h-4 w-4 shrink-0 text-primary' />
          <div className='flex-1 min-w-0'>
            <p className='text-sm font-semibold text-primary'>
              <Trans>New feature: Triggers!</Trans>
            </p>
            <p className='mt-0.5 text-sm text-black dark:text-white'>
              <Trans>Take a look at the new trigger feature on the pipeline page!</Trans>
            </p>
          </div>
          <button
            onClick={handleSeen}
            className='shrink-0 text-sm font-semibold text-primary transition-opacity hover:opacity-70'
          >
            <Trans>Seen</Trans>
          </button>
        </div>
      ),
      { id: TOAST_ID, duration: Infinity },
    );
  }, [isNew, markSeen]);

  return null;
};
