import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
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
import type { EnvEntry, Shell } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import type { PipelineNodeData } from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';
import { useSecrets } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';
import ReactCodeMirror from '@uiw/react-codemirror';
import { shell as shellLang } from '@codemirror/legacy-modes/mode/shell';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
import { StreamLanguage } from '@codemirror/language';

export type NodeFormValue =
  | { kind: 'exec'; command: string; args: string[]; workingDir?: string; env: EnvEntry[] }
  | { kind: 'script'; script: string; shell: Shell; workingDir?: string; env: EnvEntry[] };

interface StepNodeFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** If provided, the dialog is in edit mode with pre-filled values. */
  editingNode?: PipelineNodeData | null;
  onAdd: (nodeId: string, value: NodeFormValue) => void;
  onEdit: (originalId: string, nodeId: string, value: NodeFormValue) => void;
}

/** UI row keeping both a literal value and a secret ref so toggling kind never loses input. */
interface EnvRow {
  key: string;
  kind: 'literal' | 'secret';
  value: string;
  secretRef: string;
}

function rowFromEntry(entry: EnvEntry): EnvRow {
  return entry.kind === 'secret'
    ? { key: entry.key, kind: 'secret', value: '', secretRef: entry.secretRef }
    : { key: entry.key, kind: 'literal', value: entry.value, secretRef: '' };
}

function entryFromRow(row: EnvRow): EnvEntry {
  return row.kind === 'secret'
    ? { key: row.key, kind: 'secret', secretRef: row.secretRef }
    : { key: row.key, kind: 'literal', value: row.value };
}

export function StepNodeFormDialog({
  open,
  onOpenChange,
  editingNode,
  onAdd,
  onEdit,
}: StepNodeFormDialogProps) {
  const { t } = useLingui();
  const { projectId } = useParams();
  const { secrets } = useSecrets(projectId ?? '');
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
    setEnvRows(node ? node.env.map(rowFromEntry) : []);
  }, [open, editingNode]);

  const updateArg = (index: number, value: string) =>
    setArgs(prev => prev.map((a, i) => (i === index ? value : a)));
  const addArg = () => setArgs(prev => [...prev, '']);
  const removeArg = (index: number) => setArgs(prev => prev.filter((_, i) => i !== index));

  const updateEnv = (index: number, patch: Partial<EnvRow>) =>
    setEnvRows(prev => prev.map((row, i) => (i === index ? { ...row, ...patch } : row)));
  const addEnv = () =>
    setEnvRows(prev => [...prev, { key: '', kind: 'literal', value: '', secretRef: '' }]);
  const removeEnv = (index: number) => setEnvRows(prev => prev.filter((_, i) => i !== index));

  const handleSubmit = () => {
    const trimmedId = nodeId.trim();
    if (!trimmedId) return;

    const env: EnvEntry[] = envRows
      .filter(row => row.key.trim())
      .map(row => entryFromRow({ ...row, key: row.key.trim() }));
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
          <DialogTitle>
            {isEditMode ? <Trans>Edit node</Trans> : <Trans>Add a new node</Trans>}
          </DialogTitle>
          <DialogDescription>
            {isEditMode ? (
              <Trans>Modify the node properties.</Trans>
            ) : (
              <Trans>
                Define a new pipeline step. You can connect it to other nodes by dragging edges.
              </Trans>
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

                <ReactCodeMirror
                  id='node-script'
                  value={script}
                  onChange={value => setScript(value)}
                  placeholder={'cd crates/api\ncargo build --release'}
                  spellCheck={false}
                  extensions={[StreamLanguage.define(shellLang), codeMirrorTheme]}
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
                    className='flex-1'
                  />
                  <Select
                    value={row.kind}
                    onValueChange={v => updateEnv(index, { kind: v as EnvRow['kind'] })}
                  >
                    <SelectTrigger className='w-28 shrink-0'>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value='literal'>
                        <Trans>Literal</Trans>
                      </SelectItem>
                      <SelectItem value='secret'>
                        <Trans>Secret</Trans>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                  {row.kind === 'secret' ? (
                    secrets.length > 0 || row.secretRef ? (
                      <Select
                        value={row.secretRef}
                        onValueChange={v => updateEnv(index, { secretRef: v })}
                      >
                        <SelectTrigger className='flex-1'>
                          <SelectValue placeholder={t`Reference a secret`} />
                        </SelectTrigger>
                        <SelectContent>
                          {row.secretRef && !secrets.some(s => s.name === row.secretRef) && (
                            <SelectItem value={row.secretRef}>{row.secretRef}</SelectItem>
                          )}
                          {secrets.map(secret => (
                            <SelectItem key={secret.id} value={secret.name}>
                              {secret.name}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    ) : (
                      <Input
                        value={row.secretRef}
                        onChange={e => updateEnv(index, { secretRef: e.target.value })}
                        placeholder={t`secret name`}
                        className='flex-1'
                      />
                    )
                  ) : (
                    <Input
                      value={row.value}
                      onChange={e => updateEnv(index, { value: e.target.value })}
                      placeholder={t`value`}
                      className='flex-1'
                    />
                  )}
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
