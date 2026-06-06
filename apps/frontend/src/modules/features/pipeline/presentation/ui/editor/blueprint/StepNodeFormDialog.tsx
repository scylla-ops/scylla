import { useEffect, useState } from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import { Plus, Trash2 } from 'lucide-react';
import { Trans, useLingui } from '@lingui/react/macro';
import { cn } from '@shared/presentation/utils';
import type { Shell } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import type { PipelineNodeData } from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';

export type NodeFormValue =
  | { kind: 'exec'; command: string; args: string[]; workingDir?: string; env: Record<string, string> }
  | { kind: 'script'; script: string; shell: Shell; workingDir?: string; env: Record<string, string> };

interface StepNodeFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** If provided, the dialog is in edit mode with pre-filled values. */
  editingNode?: PipelineNodeData | null;
  onAdd: (nodeId: string, value: NodeFormValue) => void;
  onEdit: (originalId: string, nodeId: string, value: NodeFormValue) => void;
}

interface EnvRow {
  key: string;
  value: string;
}

const TEXTAREA_CLASS =
  'border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 dark:bg-input/30 w-full min-h-32 rounded-md border bg-transparent px-3 py-2 font-mono text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px]';

export function StepNodeFormDialog({ open, onOpenChange, editingNode, onAdd, onEdit }: StepNodeFormDialogProps) {
  const { t } = useLingui();
  const isEditMode = !!editingNode;

  const [nodeId, setNodeId] = useState('');
  const [mode, setMode] = useState<'script' | 'exec'>('script');
  const [script, setScript] = useState('');
  const [shell, setShell] = useState<Shell>('sh');
  const [command, setCommand] = useState('');
  const [args, setArgs] = useState<string[]>([]);
  const [workingDir, setWorkingDir] = useState('');
  const [envRows, setEnvRows] = useState<EnvRow[]>([]);

  // Prefill (or reset) whenever the dialog opens.
  useEffect(() => {
    if (!open) return;
    const node = editingNode;
    setNodeId(node?.id ?? '');
    setMode(node?.kind === 'exec' ? 'exec' : 'script');
    setScript(node?.kind === 'script' ? node.script : '');
    setShell(node?.kind === 'script' ? node.shell : 'sh');
    setCommand(node?.kind === 'exec' ? node.command : '');
    setArgs(node?.kind === 'exec' ? [...node.args] : []);
    setWorkingDir(node?.workingDir ?? '');
    setEnvRows(node ? Object.entries(node.env).map(([key, value]) => ({ key, value })) : []);
  }, [open, editingNode]);

  const updateArg = (index: number, value: string) =>
    setArgs(prev => prev.map((a, i) => (i === index ? value : a)));
  const addArg = () => setArgs(prev => [...prev, '']);
  const removeArg = (index: number) => setArgs(prev => prev.filter((_, i) => i !== index));

  const updateEnv = (index: number, patch: Partial<EnvRow>) =>
    setEnvRows(prev => prev.map((row, i) => (i === index ? { ...row, ...patch } : row)));
  const addEnv = () => setEnvRows(prev => [...prev, { key: '', value: '' }]);
  const removeEnv = (index: number) => setEnvRows(prev => prev.filter((_, i) => i !== index));

  const handleSubmit = () => {
    const trimmedId = nodeId.trim();
    if (!trimmedId) return;

    const env: Record<string, string> = {};
    for (const row of envRows) {
      const key = row.key.trim();
      if (key) env[key] = row.value;
    }
    const wd = workingDir.trim() ? workingDir.trim() : undefined;

    let value: NodeFormValue;
    if (mode === 'script') {
      if (!script.trim()) return;
      value = { kind: 'script', script, shell, workingDir: wd, env };
    } else {
      const trimmedCommand = command.trim();
      if (!trimmedCommand) return;
      value = {
        kind: 'exec',
        command: trimmedCommand,
        args: args.filter(a => a !== ''),
        workingDir: wd,
        env,
      };
    }

    if (isEditMode && editingNode) {
      onEdit(editingNode.id, trimmedId, value);
    } else {
      onAdd(trimmedId, value);
    }
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEditMode ? <Trans>Edit node</Trans> : <Trans>Add a new node</Trans>}</DialogTitle>
          <DialogDescription>
            {isEditMode ? (
              <Trans>Modify the node properties.</Trans>
            ) : (
              <Trans>Define a new pipeline step. You can connect it to other nodes by dragging edges.</Trans>
            )}
          </DialogDescription>
        </DialogHeader>

        <div className='space-y-4'>
          {/* Node ID */}
          <div className='space-y-2'>
            <Label htmlFor='node-id'>
              <Trans>Node ID</Trans>
            </Label>
            <Input
              id='node-id'
              value={nodeId}
              onChange={e => setNodeId(e.target.value)}
              placeholder={t`e.g., build`}
            />
          </div>

          {/* Mode toggle */}
          <div className='grid grid-cols-2 gap-1 rounded-md border p-1'>
            <button
              type='button'
              onClick={() => setMode('script')}
              className={cn(
                'rounded-sm px-3 py-1.5 text-sm font-medium transition-colors',
                mode === 'script'
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent',
              )}
            >
              <Trans>Script</Trans>
            </button>
            <button
              type='button'
              onClick={() => setMode('exec')}
              className={cn(
                'rounded-sm px-3 py-1.5 text-sm font-medium transition-colors',
                mode === 'exec'
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent',
              )}
            >
              <Trans>Command</Trans>
            </button>
          </div>

          {/* Script mode */}
          {mode === 'script' ? (
            <div className='space-y-4'>
              <div className='space-y-2'>
                <Label htmlFor='node-script'>
                  <Trans>Script</Trans>
                </Label>
                <textarea
                  id='node-script'
                  value={script}
                  onChange={e => setScript(e.target.value)}
                  placeholder={'cd crates/api\ncargo build --release'}
                  className={TEXTAREA_CLASS}
                  spellCheck={false}
                />
              </div>
              <div className='space-y-2'>
                <Label htmlFor='node-shell'>
                  <Trans>Shell</Trans>
                </Label>
                <Select value={shell} onValueChange={v => setShell(v as Shell)}>
                  <SelectTrigger id='node-shell' className='w-full'>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value='sh'>sh</SelectItem>
                    <SelectItem value='bash'>bash</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          ) : (
            /* Command mode */
            <div className='space-y-4'>
              <div className='space-y-2'>
                <Label htmlFor='node-command'>
                  <Trans>Command</Trans>
                </Label>
                <Input
                  id='node-command'
                  value={command}
                  onChange={e => setCommand(e.target.value)}
                  placeholder={t`e.g., cargo`}
                />
              </div>
              <div className='space-y-2'>
                <Label>
                  <Trans>Arguments</Trans>
                </Label>
                <div className='space-y-2'>
                  {args.map((arg, index) => (
                    <div key={index} className='flex items-center gap-2'>
                      <Input
                        value={arg}
                        onChange={e => updateArg(index, e.target.value)}
                        placeholder={t`e.g., --release`}
                      />
                      <Button
                        type='button'
                        variant='ghost'
                        size='icon-sm'
                        onClick={() => removeArg(index)}
                      >
                        <Trash2 className='size-4' />
                      </Button>
                    </div>
                  ))}
                  <Button type='button' variant='outline' size='sm' onClick={addArg}>
                    <Plus className='size-4' />
                    <Trans>Add argument</Trans>
                  </Button>
                </div>
              </div>
            </div>
          )}

          {/* Working directory */}
          <div className='space-y-2'>
            <Label htmlFor='node-working-dir'>
              <Trans>Working directory</Trans>
            </Label>
            <Input
              id='node-working-dir'
              value={workingDir}
              onChange={e => setWorkingDir(e.target.value)}
              placeholder={t`relative to the job workspace`}
            />
          </div>

          {/* Environment variables */}
          <div className='space-y-2'>
            <Label>
              <Trans>Environment variables</Trans>
            </Label>
            <div className='space-y-2'>
              {envRows.map((row, index) => (
                <div key={index} className='flex items-center gap-2'>
                  <Input
                    value={row.key}
                    onChange={e => updateEnv(index, { key: e.target.value })}
                    placeholder={t`KEY`}
                  />
                  <Input
                    value={row.value}
                    onChange={e => updateEnv(index, { value: e.target.value })}
                    placeholder={t`value`}
                  />
                  <Button
                    type='button'
                    variant='ghost'
                    size='icon-sm'
                    onClick={() => removeEnv(index)}
                  >
                    <Trash2 className='size-4' />
                  </Button>
                </div>
              ))}
              <Button type='button' variant='outline' size='sm' onClick={addEnv}>
                <Plus className='size-4' />
                <Trans>Add variable</Trans>
              </Button>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button type='button' variant='outline' onClick={() => onOpenChange(false)}>
            <Trans>Cancel</Trans>
          </Button>
          <Button type='button' onClick={handleSubmit}>
            {isEditMode ? <Trans>Save</Trans> : <Trans>Add Node</Trans>}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
