import { Trans } from '@lingui/react/macro';
import { useIsUnseen } from '@/modules/layout/presentation/hooks/use-whats-new.ts';

interface NewBadgeProps {
  /** Highlight announced here — the badge disappears once it is marked seen. */
  highlightId: string;
}

/** Pulsing "New" pill for a sidebar entry this release introduces. */
export const NewBadge = ({ highlightId }: NewBadgeProps) => {
  const isUnseen = useIsUnseen(highlightId);

  if (!isUnseen) return null;

  return (
    // Hidden on the icon rail: there is no room for a pill beside a 1rem icon.
    <span className='relative ml-auto inline-flex group-data-[collapsible=icon]:hidden'>
      <span className='absolute inset-0 animate-ping rounded-full bg-primary opacity-40' />
      <span className='relative rounded-full bg-primary px-1.5 py-0.5 text-[10px] font-bold leading-none text-primary-foreground'>
        <Trans>New</Trans>
      </span>
    </span>
  );
};
