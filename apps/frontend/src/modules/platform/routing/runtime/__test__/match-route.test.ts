// @vitest-environment node
import { describe, it, expect } from 'vitest';
import type { CompiledRoute } from '../../compilation/compile-routes.ts';
import { splitPath } from '../../compilation/route-path.ts';
import { matchRoute } from '../match-route.ts';

const route = (path: string): CompiledRoute => ({
  mount: 'organization',
  path: splitPath(path),
  wrappers: [],
  trail: [],
});

const routes = [
  route('/login'),
  route('/:slug/agents'),
  route('/:slug/agents/:agentId'),
  route('/:slug'),
];

describe('matchRoute', () => {
  it('finds the route whose segments fit, and reads its parameters', () => {
    const match = matchRoute(routes, '/acme/agents/agent-1');

    expect(match?.route).toBe(routes[2]);
    expect(match?.params).toEqual({ slug: 'acme', agentId: 'agent-1' });
  });

  it('takes the first fit, which the compiled order makes the most specific', () => {
    expect(matchRoute(routes, '/login')?.route).toBe(routes[0]);
  });

  it('needs every segment to fit, no more and no fewer', () => {
    expect(matchRoute(routes, '/acme/agents/agent-1/logs')).toBeNull();
    expect(matchRoute(routes, '/')).toBeNull();
  });

  it('ignores a trailing slash', () => {
    expect(matchRoute(routes, '/acme/agents/')?.route).toBe(routes[1]);
  });

  it('decodes a parameter, and keeps one that is not valid encoding as it is', () => {
    expect(matchRoute(routes, '/acme/agents/a%20b')?.params.agentId).toBe('a b');
    expect(matchRoute(routes, '/acme/agents/100%')?.params.agentId).toBe('100%');
  });
});
