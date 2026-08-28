import { cn } from '@shared/presentation/utils';
import React, { type ReactNode } from 'react';
import { SeparatorVertical } from 'lucide-react';

export type ListCardSection = {
  content: ReactNode;
  className?: string;
  width?: string;
  noSeparator?: boolean;
};

export type ListCardProps = {
  sections: ListCardSection[];
  onClick?: () => void;
  selected?: boolean;
  className?: string;
  height?: string;
};

/**
 * Component to display a list card with sections and separators.
 */
export const ListCard = ({ sections, onClick, selected, className, height }: ListCardProps) => {
  return (
    <div
      onClick={onClick}
      className={cn(
        'w-full group flex items-center gap-4 p-4 rounded-xl border transition-all duration-200 overflow-hidden',
        onClick && 'cursor-pointer',
        !selected &&
          !className?.includes('hover:') && [
            'bg-white border-slate-200',
            'hover:border-slate-300 hover:shadow-sm', // Ces hovers ne s'appliquent que si...
          ],
        selected && 'bg-blue-200 border-blue-400',
        className,
        height ? { height } : { height: '60px' },
      )}
    >
      {sections.map((section, index) => {
        const isLast = index === sections.length - 1;
        const showSeparator = !isLast && !section.noSeparator;

        return (
          <React.Fragment key={index}>
            <div
              className={cn('min-w-0', section.className)}
              style={section.width ? { width: section.width } : undefined}
            >
              {section.content}
            </div>
            {showSeparator && (
              <SeparatorVertical className='m-3 h-9/12 w-px bg-slate-200 shrink-0' />
            )}
          </React.Fragment>
        );
      })}
    </div>
  );
};
