import { useNavigate } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { Copy, KeyRound, MoreHorizontal, Trash } from 'lucide-react';
import { Badge, Button, Card, CardContent } from '@shadcn';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@shadcn/dropdown-menu.tsx';
import type { App } from '@/modules/features/apps/domain/models/app.model.ts';
import { formatDate } from '@shared/utils/date-utils.ts';
import { toast } from 'sonner';

interface AppCardProps {
  app: App;
  onRequestDelete: (id: string) => void;
}

export const AppCard = ({ app, onRequestDelete }: AppCardProps) => {
  const navigate = useNavigate();

  const copyId = async () => {
    await navigator.clipboard.writeText(app.id);
    toast.success('App id copied');
  };

  return (
    <Card
      className='cursor-pointer gap-0 py-0 transition-colors hover:bg-accent/50'
      onClick={() => navigate(app.id)}
    >
      <CardContent className='p-4'>
        <div className='flex items-start justify-between gap-2'>
          <div className='flex min-w-0 items-center gap-3'>
            <span className='flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-success/30 bg-success/10'>
              <KeyRound className='h-4 w-4 text-success' />
            </span>
            <div className='min-w-0'>
              <p className='truncate font-semibold'>{app.name}</p>
              <p className='truncate font-mono text-xs text-muted-foreground'>{app.id}</p>
            </div>
          </div>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='ghost'
                size='icon'
                className='h-7 w-7 shrink-0'
                onClick={e => e.stopPropagation()}
              >
                <MoreHorizontal className='h-4 w-4' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end' onClick={e => e.stopPropagation()}>
              <DropdownMenuItem onClick={copyId}>
                <Copy className='mr-2 h-4 w-4' />
                <Trans>Copy id</Trans>
              </DropdownMenuItem>
              <DropdownMenuItem variant='destructive' onClick={() => onRequestDelete(app.id)}>
                <Trash className='mr-2 h-4 w-4' />
                <Trans>Delete</Trans>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        {!app.isActive && (
          <Badge variant='secondary' className='mt-3'>
            <Trans>inactive</Trans>
          </Badge>
        )}
      </CardContent>

      <div className='flex items-center justify-between border-t border-dashed px-4 py-2.5'>
        <span className='text-xs text-muted-foreground'>
          <Trans>created</Trans> {formatDate(app.createdAt)}
        </span>
        <Button
          variant='ghost'
          size='icon'
          className='h-7 w-7 text-muted-foreground hover:text-destructive'
          onClick={e => {
            e.stopPropagation();
            onRequestDelete(app.id);
          }}
        >
          <Trash className='h-4 w-4' />
        </Button>
      </div>
    </Card>
  );
};
