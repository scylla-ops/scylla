// @vitest-environment node
import { describe, expect, it } from 'vitest';
import { resolveTarget } from './resolve-target.ts';

describe('resolveTarget', () => {
  it('keeps an absolute target as it is', () => {
    expect(resolveTarget('/acme/projects', '/globex/dashboard')).toEqual({
      pathname: '/acme/projects',
      search: '',
      hash: '',
    });
  });

  it('goes to the parent page for ..', () => {
    expect(resolveTarget('..', '/acme/agents/agent-1').pathname).toBe('/acme/agents');
  });

  it('goes to a child page for a relative segment', () => {
    expect(resolveTarget('members', '/acme/projects/p1').pathname).toBe(
      '/acme/projects/p1/members',
    );
  });

  it('ignores a trailing slash on the current pathname', () => {
    expect(resolveTarget('..', '/acme/agents/agent-1/').pathname).toBe('/acme/agents');
  });

  it('splits the query string and the hash from the pathname', () => {
    expect(resolveTarget('/acme/jobs/j1?nodes=build,test#logs', '/')).toEqual({
      pathname: '/acme/jobs/j1',
      search: '?nodes=build,test',
      hash: '#logs',
    });
  });

  it('keeps the root as a single slash', () => {
    expect(resolveTarget('/', '/acme').pathname).toBe('/');
  });
});
