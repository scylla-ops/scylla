import type { Component } from 'svelte';
import type { LucideProps } from '@lucide/svelte';

/**
 * Any icon from `@lucide/svelte`, as a prop type.
 *
 * The React twin imports `LucideIcon` from `lucide-react`; the Svelte package
 * exports no such alias, only the props, so it is spelled out once here rather
 * than in every component that takes an icon.
 */
export type LucideIcon = Component<LucideProps>;
