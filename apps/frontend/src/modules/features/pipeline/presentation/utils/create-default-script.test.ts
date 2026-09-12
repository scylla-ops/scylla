import { describe, it, expect } from 'vitest';
import { createDefaultScript } from './create-default-script';

describe('createDefaultScript', () => {
  it('produces valid, pretty-printed JSON scoped to the given project', () => {
    const script = createDefaultScript('project-1');
    expect(script).toContain('\n'); // pretty-printed (indent: 2)

    const parsed = JSON.parse(script) as { name: string; projectId: string; nodes: unknown[] };
    expect(parsed.projectId).toBe('project-1');
    expect(parsed.name).toBe('my-pipeline');
    expect(parsed.nodes).toHaveLength(2);
  });

  it('the second node depends on the first, so the example runs after the welcome message', () => {
    const parsed = JSON.parse(createDefaultScript('p')) as {
      nodes: { id: string; deps: string[] }[];
    };
    expect(parsed.nodes[0]).toMatchObject({ id: 'welcome', deps: [] });
    expect(parsed.nodes[1]).toMatchObject({ id: 'example', deps: ['welcome'] });
  });

  it('is deterministic for the same projectId (differs only by that id)', () => {
    expect(createDefaultScript('a')).not.toBe(createDefaultScript('b'));
    expect(createDefaultScript('a')).toBe(createDefaultScript('a'));
  });
});
