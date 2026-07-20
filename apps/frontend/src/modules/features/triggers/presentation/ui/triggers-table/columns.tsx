import type { ColumnDef } from '@tanstack/react-table';
import { Clock, HelpCircle, Webhook } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { Switch } from '@shadcn/switch.tsx';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { TriggerSourceCell } from './TriggerSourceCell.tsx';
import { TriggerStatusCell } from './TriggerStatusCell.tsx';
import { TriggerActions } from './TriggerActions.tsx';

const SOURCE_ICON = {
  [TriggerKind.Cron]: Clock,
  [TriggerKind.Webhook]: Webhook,
  [TriggerKind.Unknown]: HelpCircle,
} as const;

interface TriggerColumnsMeta {
  onFire: (trigger: TriggerEntity) => void;
  onEdit: (trigger: TriggerEntity) => void;
  onDelete: (triggerId: string) => void;
  onToggleEnabled: (trigger: TriggerEntity, enabled: boolean) => void;
  firingIds: Set<string>;
}

export const createTriggerColumns = (meta: TriggerColumnsMeta): ColumnDef<TriggerEntity>[] => [
  {
    id: 'name',
    header: () => <Trans>Name</Trans>,
    cell: ({ row }) => {
      const trigger = row.original;
      const Icon = SOURCE_ICON[trigger.source.kind];
      return (
        <div className='flex flex-row items-center gap-3'>
          <div className='flex size-10 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
            <Icon className='size-4 text-primary' />
          </div>
          <div className='flex flex-col items-start'>
            <p className='font-semibold text-foreground truncate'>{trigger.name}</p>
            <p className='font-mono text-xs text-muted-foreground truncate'>ID: {trigger.id}</p>
          </div>
        </div>
      );
    },
    size: 240,
  },
  {
    id: 'source',
    header: () => <Trans>Source</Trans>,
    cell: ({ row }) => <TriggerSourceCell trigger={row.original} />,
    size: 260,
  },
  {
    id: 'status',
    header: () => <Trans>Status</Trans>,
    cell: ({ row }) => <TriggerStatusCell trigger={row.original} />,
    size: 180,
  },
  {
    id: 'enabled',
    header: () => <Trans>Enabled</Trans>,
    cell: ({ row }) => {
      const trigger = row.original;
      return (
        // Stop propagation so toggling the switch doesn't also select the row.
        <div className='flex justify-center' onClick={e => e.stopPropagation()}>
          <Switch
            checked={trigger.enabled}
            onCheckedChange={checked => meta.onToggleEnabled(trigger, checked)}
          />
        </div>
      );
    },
    size: 100,
  },
  {
    id: 'actions',
    header: () => <Trans>Actions</Trans>,
    cell: ({ row }) => {
      const trigger = row.original;
      return (
        <TriggerActions
          isFiring={meta.firingIds.has(trigger.id)}
          onFire={e => {
            e.stopPropagation();
            meta.onFire(trigger);
          }}
          onEdit={e => {
            e.stopPropagation();
            meta.onEdit(trigger);
          }}
          onDelete={e => {
            e.stopPropagation();
            meta.onDelete(trigger.id);
          }}
        />
      );
    },
    size: 140,
  },
];
