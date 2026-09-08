import { useRef, useState, type ReactNode } from 'react';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { cn } from '@shared/presentation/utils';

interface TruncatedTextProps {
  children: ReactNode;
  /** Tooltip body; defaults to `children`. */
  tooltip?: ReactNode;
  className?: string;
}

/**
 * Single-line text that ellipsizes, and reveals itself in a tooltip *only* when it
 * is actually cut off — a tooltip repeating text already fully visible is noise.
 *
 * The overflow is measured when the pointer arrives rather than watched with a
 * ResizeObserver: a resize while nothing hovers the text changes nothing the user
 * can see, and this keeps the component free of subscriptions in every table row.
 *
 * Needs a bounded parent to have any effect — a flex/grid ancestor with `min-w-0`.
 */
export const TruncatedText = ({ children, tooltip, className }: TruncatedTextProps) => {
  const textRef = useRef<HTMLSpanElement>(null);
  const [isTruncated, setIsTruncated] = useState(false);

  const measure = () => {
    const element = textRef.current;
    if (element) setIsTruncated(element.scrollWidth > element.clientWidth);
  };

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span
          ref={textRef}
          onPointerEnter={measure}
          onFocus={measure}
          className={cn('block min-w-0 truncate', className)}
        >
          {children}
        </span>
      </TooltipTrigger>
      {isTruncated && <TooltipContent>{tooltip ?? children}</TooltipContent>}
    </Tooltip>
  );
};
