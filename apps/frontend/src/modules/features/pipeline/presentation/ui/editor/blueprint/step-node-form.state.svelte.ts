import type {
  EnvEntry,
  PipelineStep,
  Shell,
} from '../../../../domain/structs/pipeline.struct.ts';
import type { NodeFormValue } from '../../../utils/blueprint-converter.ts';

/** Keeps both a literal and a secret reference, so switching the kind loses nothing typed. */
export interface EnvRow {
  key: string;
  kind: 'literal' | 'secret';
  value: string;
  secretRef: string;
}

const rowFrom = (entry: EnvEntry): EnvRow =>
  entry.kind === 'secret'
    ? { key: entry.key, kind: 'secret', value: '', secretRef: entry.secretRef }
    : { key: entry.key, kind: 'literal', value: entry.value, secretRef: '' };

const entryFrom = (row: EnvRow): EnvEntry =>
  row.kind === 'secret'
    ? { key: row.key, kind: 'secret', secretRef: row.secretRef }
    : { key: row.key, kind: 'literal', value: row.value };

/** Seeded once: the dialog rebuilds it at each opening. `toValue()` holds every rule of a valid step. */
export const createStepNodeForm = (editingStep?: PipelineStep) => {
  let nodeId = $state(editingStep?.id ?? '');
  let mode = $state<'script' | 'exec'>(editingStep?.kind === 'exec' ? 'exec' : 'script');
  let script = $state(editingStep?.kind === 'script' ? editingStep.script : '');
  let shell = $state<Shell>(editingStep?.kind === 'script' ? editingStep.shell : 'sh');
  let command = $state(editingStep?.kind === 'exec' ? editingStep.command : '');
  let args = $state<string[]>(editingStep?.kind === 'exec' ? [...editingStep.args] : []);
  let workingDir = $state(editingStep?.workingDir ?? '');
  let envRows = $state<EnvRow[]>(editingStep ? editingStep.env.map(rowFrom) : []);

  return {
    get nodeId() {
      return nodeId;
    },
    set nodeId(next: string) {
      nodeId = next;
    },
    get mode() {
      return mode;
    },
    set mode(next: 'script' | 'exec') {
      mode = next;
    },
    get script() {
      return script;
    },
    set script(next: string) {
      script = next;
    },
    get shell() {
      return shell;
    },
    set shell(next: Shell) {
      shell = next;
    },
    get command() {
      return command;
    },
    set command(next: string) {
      command = next;
    },
    get workingDir() {
      return workingDir;
    },
    set workingDir(next: string) {
      workingDir = next;
    },

    // An argument is its position: indexed, like its `{#each}` key.
    get args() {
      return args;
    },
    setArg: (index: number, value: string) => {
      args = args.map((arg, i) => (i === index ? value : arg));
    },
    addArg: () => {
      args = [...args, ''];
    },
    removeArg: (index: number) => {
      args = args.filter((_, i) => i !== index);
    },

    get envRows() {
      return envRows;
    },
    setEnv: (index: number, patch: Partial<EnvRow>) => {
      envRows = envRows.map((row, i) => (i === index ? { ...row, ...patch } : row));
    },
    addEnv: () => {
      envRows = [...envRows, { key: '', kind: 'literal', value: '', secretRef: '' }];
    },
    removeEnv: (index: number) => {
      envRows = envRows.filter((_, i) => i !== index);
    },

    /** `null` when the step cannot run (no id, no script, no command). */
    toValue(): { nodeId: string; value: NodeFormValue } | null {
      const trimmedId = nodeId.trim();
      if (!trimmedId) return null;

      // A row with no key is an empty line.
      const env = envRows
        .filter(row => row.key.trim())
        .map(row => entryFrom({ ...row, key: row.key.trim() }));
      const directory = workingDir.trim() ? workingDir.trim() : undefined;

      if (mode === 'script') {
        if (!script.trim()) return null;
        return {
          nodeId: trimmedId,
          value: { kind: 'script', script, shell, workingDir: directory, env },
        };
      }

      const trimmedCommand = command.trim();
      if (!trimmedCommand) return null;

      return {
        nodeId: trimmedId,
        value: {
          kind: 'exec',
          command: trimmedCommand,
          // Blank rows are unfilled lines, not empty arguments.
          args: args.filter(arg => arg !== ''),
          workingDir: directory,
          env,
        },
      };
    },
  };
};

export type StepNodeForm = ReturnType<typeof createStepNodeForm>;
