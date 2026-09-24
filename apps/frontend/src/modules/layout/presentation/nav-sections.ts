import type { Snippet } from 'svelte';
import type { MessageDescriptor } from '@lingui/core';
import type { Permission } from '@platform/authz';
import type { NavEntry } from '@platform/routing';
import type { NavSection } from './structs/nav-section.struct.ts';

export interface NavSectionsInput {
  entries: readonly NavEntry[];
  /** E.g. `/acme`. */
  prefix: string;
  can: (permission: Permission) => boolean;
  translate: (message: MessageDescriptor) => string;
  highlightIdFor: (url: string) => string | undefined;
  titles: { organization: string; system: string };
  organizationHeader?: Snippet;
}

/** Hides the entries the user cannot open, then the empty sections (a section with a header stays). */
export const navSectionsFor = ({
  entries,
  prefix,
  can,
  translate,
  highlightIdFor,
  titles,
  organizationHeader,
}: NavSectionsInput): NavSection[] => {
  const itemsOf = (section: NavEntry['section']) =>
    entries
      .filter(entry => entry.section === section)
      .filter(entry => !entry.permission || can(entry.permission))
      .map(entry => ({
        title: translate(entry.title),
        url: `${prefix}/${entry.url}`,
        icon: entry.icon,
        highlightId: highlightIdFor(entry.url),
      }));

  const sections: NavSection[] = [
    { title: titles.organization, header: organizationHeader, items: itemsOf('organization') },
    { title: titles.system, items: itemsOf('system') },
  ];

  return sections.filter(section => section.items.length > 0 || section.header);
};
