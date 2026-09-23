// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { createRawSnippet } from 'svelte';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz';
import type { NavEntry } from '@platform/routing';
import { navSectionsFor, type NavSectionsInput } from './nav-sections.ts';

const entry = (overrides: Partial<NavEntry> = {}): NavEntry => ({
  section: 'organization',
  title: msg`Secrets`,
  url: 'secrets',
  permission: Permission.LIST_SECRETS,
  ...overrides,
});

const header = createRawSnippet(() => ({ render: () => '<span>selector</span>' }));

const sectionsFor = (
  entries: readonly NavEntry[],
  held: readonly Permission[],
  overrides: Partial<NavSectionsInput> = {},
) =>
  navSectionsFor({
    entries,
    prefix: '/acme-corp',
    can: permission => held.includes(permission),
    translate: message => message.message ?? message.id,
    highlightIdFor: url => (url === 'secrets' ? 'secrets-highlight' : undefined),
    titles: { organization: 'Organization', system: 'System' },
    organizationHeader: header,
    ...overrides,
  });

const titles = (sections: ReturnType<typeof sectionsFor>) =>
  sections.flatMap(section => section.items.map(item => item.title));

describe('navSectionsFor', () => {
  it('keeps an entry whose permission the user holds', () => {
    expect(titles(sectionsFor([entry()], [Permission.LIST_SECRETS]))).toEqual(['Secrets']);
  });

  it('removes an entry whose permission the user lacks', () => {
    expect(titles(sectionsFor([entry()], [Permission.READ_PROJECT]))).toEqual([]);
  });

  it('always keeps an entry that declares no permission', () => {
    const entries = [entry({ permission: undefined, title: msg`Dashboard`, url: 'dashboard' })];
    expect(titles(sectionsFor(entries, []))).toEqual(['Dashboard']);
  });

  it('filters per entry, not per section', () => {
    const entries = [
      entry(),
      entry({ title: msg`Pipelines`, url: 'pipelines', permission: Permission.LIST_PIPELINES }),
    ];
    expect(titles(sectionsFor(entries, [Permission.LIST_SECRETS]))).toEqual(['Secrets']);
  });

  it('drops a section left empty, but keeps the organization section for its header', () => {
    const entries = [entry(), entry({ section: 'system', title: msg`Users`, url: 'users' })];
    const sections = sectionsFor(entries, []);

    expect(sections.map(section => section.title)).toEqual(['Organization']);
    expect(sections[0].header).toBe(header);
  });

  it('keeps a system entry the user holds', () => {
    const entries = [
      entry({ section: 'system', title: msg`Users`, url: 'users', permission: Permission.LIST_USERS }),
    ];
    const sections = sectionsFor(entries, [Permission.LIST_USERS]);

    expect(sections.find(section => section.title === 'System')?.items.map(i => i.title)).toEqual([
      'Users',
    ]);
  });

  it('prefixes the entry urls with the current organization', () => {
    const [organization] = sectionsFor([entry()], [Permission.LIST_SECRETS]);
    expect(organization.items[0].url).toBe('/acme-corp/secrets');
  });

  it('gives an entry the release highlight that announces its url', () => {
    const [organization] = sectionsFor([entry()], [Permission.LIST_SECRETS]);
    expect(organization.items[0].highlightId).toBe('secrets-highlight');
  });
});
