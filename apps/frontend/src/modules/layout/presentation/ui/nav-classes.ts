import { cn } from '@shared/presentation/utils/cn.ts';

/** On the icon rail, the card shrinks around the icon column. */
export const NAV_SECTION_CARD_CLASS = cn(
  'gap-1 rounded-xl border border-border bg-background p-2 shadow-sm',
  'group-data-[collapsible=icon]:mx-auto group-data-[collapsible=icon]:w-fit',
  'group-data-[collapsible=icon]:rounded-lg group-data-[collapsible=icon]:p-1',
);

export const NAV_SECTION_DIVIDER_CLASS = 'my-1 border-t border-border';
