import { quintOut } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';
import { motionDuration } from '@shared/presentation/ui';

/** The entrance of the welcome screen: it fades in, grows and moves up. */
export const welcomeIn = (_node?: Element): TransitionConfig => ({
  duration: motionDuration(800),
  easing: quintOut,
  css: t => `opacity: ${t}; transform: translateY(${(1 - t) * 1.25}rem) scale(${0.95 + 0.05 * t})`,
});
