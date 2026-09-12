import { describe, it, expect } from 'vitest';
import { GrpcPipelineMapper } from './grpc-pipeline.mapper';
import { Shell } from '@/generated/scylla/exec/v1/step.ts';
import type { Pipeline, PipelineNode, PipelineSummary, EnvVar } from '@/generated/scylla/pipeline/v1/pipeline.ts';
import type { PipelineStep, EnvEntry } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

const TS = { seconds: 1735689600n, nanos: 0 };
const ISO = '2025-01-01T00:00:00.000Z';

const execNode = (overrides: Partial<PipelineNode> = {}): PipelineNode => ({
  nodeId: { value: 'node-1' },
  deps: [],
  workingDir: '',
  env: [],
  step: { oneofKind: 'exec', exec: { command: 'echo', args: ['hi'] } },
  ...overrides,
});

const basePipeline = (overrides: Partial<Pipeline> = {}): Pipeline => ({
  pipelineId: { value: 'pipeline-1' },
  projectId: { value: 'project-1' },
  name: 'ci',
  nodes: [],
  createdAt: TS,
  updatedAt: TS,
  ...overrides,
});

describe('GrpcPipelineMapper.toDomain (via nodeToDomain)', () => {
  it('unwraps pipeline-level ids and carries the name through', () => {
    const domain = GrpcPipelineMapper.toDomain(basePipeline());
    expect(domain.id).toBe('pipeline-1');
    expect(domain.projectId).toBe('project-1');
    expect(domain.name).toBe('ci');
  });

  it('maps an exec step', () => {
    const [step] = GrpcPipelineMapper.toDomain(basePipeline({ nodes: [execNode()] })).nodes;
    expect(step).toMatchObject({ kind: 'exec', command: 'echo', args: ['hi'] });
  });

  it('maps a script step, including the shell', () => {
    const [step] = GrpcPipelineMapper.toDomain(
      basePipeline({
        nodes: [execNode({ step: { oneofKind: 'script', script: { script: 'echo hi', shell: Shell.BASH } } })],
      }),
    ).nodes;
    expect(step).toMatchObject({ kind: 'script', script: 'echo hi', shell: 'bash' });
  });

  it('SHELL_SH and SHELL_UNSPECIFIED both read as "sh" (the agent treats unspecified as sh)', () => {
    const shellOf = (shell: Shell) =>
      GrpcPipelineMapper.toDomain(
        basePipeline({ nodes: [execNode({ step: { oneofKind: 'script', script: { script: 's', shell } } })] }),
      ).nodes[0];

    expect(shellOf(Shell.SH)).toMatchObject({ shell: 'sh' });
    expect(shellOf(Shell.UNSPECIFIED)).toMatchObject({ shell: 'sh' });
  });

  it('a step oneof this build does not understand throws, rather than silently dropping it', () => {
    expect(() =>
      GrpcPipelineMapper.toDomain(
        basePipeline({ nodes: [execNode({ step: { oneofKind: undefined } })] }),
      ),
    ).toThrow(/doesn't understand/);
  });

  it('an empty workingDir is omitted from the domain step entirely', () => {
    const [step] = GrpcPipelineMapper.toDomain(basePipeline({ nodes: [execNode({ workingDir: '' })] })).nodes;
    expect(step).not.toHaveProperty('workingDir');
  });

  it('a non-empty workingDir is carried through', () => {
    const [step] = GrpcPipelineMapper.toDomain(
      basePipeline({ nodes: [execNode({ workingDir: '/srv/app' })] }),
    ).nodes;
    expect(step.workingDir).toBe('/srv/app');
  });

  it('unwraps each dep id', () => {
    const [step] = GrpcPipelineMapper.toDomain(
      basePipeline({ nodes: [execNode({ deps: [{ value: 'a' }, { value: 'b' }] })] }),
    ).nodes;
    expect(step.deps).toEqual(['a', 'b']);
  });

  describe('env', () => {
    it('maps a literal (value) env var', () => {
      const [step] = GrpcPipelineMapper.toDomain(
        basePipeline({ nodes: [execNode({ env: [{ key: 'K', source: { oneofKind: 'value', value: 'v' } }] })] }),
      ).nodes;
      expect(step.env).toEqual([{ key: 'K', kind: 'literal', value: 'v' }]);
    });

    it('maps a secretRef env var', () => {
      const [step] = GrpcPipelineMapper.toDomain(
        basePipeline({
          nodes: [execNode({ env: [{ key: 'K', source: { oneofKind: 'secretRef', secretRef: 'db-url' } }] })],
        }),
      ).nodes;
      expect(step.env).toEqual([{ key: 'K', kind: 'secret', secretRef: 'db-url' }]);
    });

    it('an env source oneof this build does not understand throws', () => {
      expect(() =>
        GrpcPipelineMapper.toDomain(
          basePipeline({ nodes: [execNode({ env: [{ key: 'K', source: { oneofKind: undefined } }] })] }),
        ),
      ).toThrow(/doesn't understand/);
    });
  });
});

describe('GrpcPipelineMapper.nodeFromDomain', () => {
  const execStep = (params: { command?: string; args?: string[]; deps?: string[]; env?: EnvEntry[] } = {}): PipelineStep => ({
    id: 'node-1',
    deps: params.deps ?? [],
    env: params.env ?? [],
    kind: 'exec',
    command: params.command ?? 'echo',
    args: params.args ?? ['hi'],
  });

  const scriptStep = (shell: 'sh' | 'bash'): PipelineStep => ({
    id: 'node-1',
    deps: [],
    env: [],
    kind: 'script',
    script: 'echo hi',
    shell,
  });

  it('defaults workingDir to "" when the domain step has none', () => {
    const node = GrpcPipelineMapper.nodeFromDomain(execStep());
    expect(node.workingDir).toBe('');
  });

  it('wraps the id and every dep id', () => {
    const node = GrpcPipelineMapper.nodeFromDomain(execStep({ deps: ['a', 'b'] } ));
    expect(node.nodeId).toEqual({ value: 'node-1' });
    expect(node.deps).toEqual([{ value: 'a' }, { value: 'b' }]);
  });

  it('builds an exec step', () => {
    const node = GrpcPipelineMapper.nodeFromDomain(execStep({ command: 'ls', args: ['-la'] }));
    expect(node.step).toEqual({ oneofKind: 'exec', exec: { command: 'ls', args: ['-la'] } });
  });

  it('builds a script step, mapping "bash" to Shell.BASH and anything else to Shell.SH', () => {
    const bashNode = GrpcPipelineMapper.nodeFromDomain(scriptStep('bash'));
    expect(bashNode.step).toEqual({ oneofKind: 'script', script: { script: 'echo hi', shell: Shell.BASH } });

    const shNode = GrpcPipelineMapper.nodeFromDomain(scriptStep('sh'));
    expect(shNode.step).toEqual({ oneofKind: 'script', script: { script: 'echo hi', shell: Shell.SH } });
  });

  describe('env', () => {
    const withEnv = (env: EnvEntry[]) => GrpcPipelineMapper.nodeFromDomain(execStep({ env }));

    it('maps a literal entry to the value oneof', () => {
      const node = withEnv([{ key: 'K', kind: 'literal', value: 'v' }]);
      expect(node.env).toEqual([{ key: 'K', source: { oneofKind: 'value', value: 'v' } } satisfies EnvVar]);
    });

    it('maps a secret entry to the secretRef oneof', () => {
      const node = withEnv([{ key: 'K', kind: 'secret', secretRef: 'db-url' }]);
      expect(node.env).toEqual([
        { key: 'K', source: { oneofKind: 'secretRef', secretRef: 'db-url' } } satisfies EnvVar,
      ]);
    });
  });
});

describe('GrpcPipelineMapper.toDomainInfoList', () => {
  it('maps pipeline summaries, including timestamps, and carries pagination through', () => {
    const summary: PipelineSummary = {
      pipelineId: { value: 'pipeline-1' },
      projectId: { value: 'project-1' },
      name: 'ci',
      nodeCount: 3,
      createdAt: TS,
      updatedAt: TS,
    };
    const pagination = { totalCount: 1, page: 1, pageSize: 10, totalPages: 1, hasNext: false, hasPrevious: false };

    const result = GrpcPipelineMapper.toDomainInfoList({ pipelines: [summary], pagination });

    expect(result.items).toEqual([
      { id: 'pipeline-1', projectId: 'project-1', name: 'ci', nodeCount: 3, createdAt: ISO, updatedAt: ISO },
    ]);
    expect(result.pagination).toBe(pagination);
  });
});
