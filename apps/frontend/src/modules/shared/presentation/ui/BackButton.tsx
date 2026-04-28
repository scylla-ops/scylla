import type { ComponentProps, ReactNode } from 'react';
import { Button } from '@shadcn';
import { ArrowLeft } from 'lucide-react';

type BackButtonProps = Omit<ComponentProps<typeof Button>, 'className'> & {
  label?: ReactNode;
  iconOnly?: boolean;
  className?: string;
};

export const BackButton = ({
  label = 'Back',
  iconOnly = false,
  className = '',
  ...props
}: BackButtonProps) => {
  const buttonClassName = iconOnly
    ? `h-8 w-8 ${className}`.trim()
    : `flex items-center gap-2 ${className}`.trim();

  return (
    <Button
      {...props}
      variant={props.variant ?? 'ghost'}
      size={iconOnly ? 'icon' : props.size}
      className={buttonClassName}
    >
      <ArrowLeft className='size-4' />
      {!iconOnly && label}
    </Button>
  );
};
