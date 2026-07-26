import { Button } from '@shadcn';
import { TabsList, TabsTrigger } from '@shadcn/tabs.tsx';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Trans } from '@lingui/react/macro';
import { Loader2 } from 'lucide-react';
import { AnimatePresence, motion } from 'framer-motion';
import type { ReactNode } from 'react';
import { cn } from '@shared/presentation/utils';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

interface PipelineEditorHeaderProps {
  onSubmit: () => void;
  submitLabel: string;
  /** `create` shows a neutral draft state; `edit` tracks dirty/saved divergence. */
  mode: 'create' | 'edit';
  /** Disables the submit button (e.g. when the script JSON is invalid). */
  submitDisabled?: boolean;
  /** Disables the blueprint tab so an invalid script can't be silently overwritten. */
  blueprintDisabled?: boolean;
  isDirty: boolean;
  isSaving?: boolean;
}

type SaveStatus = 'draft' | 'saving' | 'dirty' | 'saved';

const STATUS_LABEL: Record<SaveStatus, ReactNode> = {
  draft: <Trans>Draft — not created yet</Trans>,
  saving: <Trans>Saving…</Trans>,
  dirty: <Trans>Unsaved changes</Trans>,
  saved: <Trans>All changes saved</Trans>,
};

export const PipelineEditorHeader = ({
  onSubmit,
  submitLabel,
  mode,
  submitDisabled,
  blueprintDisabled,
  isDirty,
  isSaving = false,
}: PipelineEditorHeaderProps) => {
  const status: SaveStatus = isSaving
    ? 'saving'
    : mode === 'create'
      ? 'draft'
      : isDirty
        ? 'dirty'
        : 'saved';

  // Creating vs. editing needs a different permission, in the current project context.
  const canSubmit = useCan(
    mode === 'create' ? Permission.CREATE_PIPELINE : Permission.UPDATE_PIPELINE,
  );

  return (
    <div className={'flex w-full items-center justify-between gap-3'}>
      <TabsList>
        <TabsTrigger value='scripting'>
          <Trans>Scripting</Trans>
        </TabsTrigger>
        <TabsTrigger value='blueprint' disabled={blueprintDisabled}>
          <Trans>Blueprint</Trans>
        </TabsTrigger>
      </TabsList>

      <div className='flex items-center gap-3'>
        <div className='flex items-center gap-2 text-sm text-muted-foreground'>
          <span className='relative flex size-2'>
            {status === 'dirty' && (
              <span className='absolute inline-flex size-full animate-ping rounded-full bg-amber-500/70' />
            )}
            <span
              className={cn(
                'relative inline-flex size-2 rounded-full transition-colors duration-300',
                (status === 'draft' || status === 'saving') && 'bg-muted-foreground/60',
                status === 'dirty' && 'bg-amber-500',
                status === 'saved' && 'bg-emerald-500',
              )}
            />
          </span>
          <AnimatePresence mode='wait' initial={false}>
            <motion.span
              key={status}
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              transition={{ duration: 0.15, ease: 'easeOut' }}
              className='hidden whitespace-nowrap md:inline-flex'
            >
              {STATUS_LABEL[status]}
            </motion.span>
          </AnimatePresence>
        </div>

        {canSubmit ? (
          <Button onClick={onSubmit} disabled={submitDisabled || isSaving}>
            {isSaving && <Loader2 className='mr-2 size-4 animate-spin' />}
            <Trans>{submitLabel}</Trans>
          </Button>
        ) : (
          <Tooltip>
            <TooltipTrigger asChild>
              <span className='inline-flex'>
                <Button disabled className='pointer-events-none'>
                  <Trans>{submitLabel}</Trans>
                </Button>
              </span>
            </TooltipTrigger>
            <TooltipContent>
              <p>
                {mode === 'create' ? (
                  <Trans>You don't have permission to create pipelines.</Trans>
                ) : (
                  <Trans>You don't have permission to edit this pipeline.</Trans>
                )}
              </p>
            </TooltipContent>
          </Tooltip>
        )}
      </div>
    </div>
  );
};
