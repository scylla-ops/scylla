import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { messages } from '@/modules/features/dashboard/locales/fr/messages.ts';
import { render } from '@/test/render.svelte.ts';
import { withLocale } from '@/test/i18n.ts';
import type { JobsSummary } from '@/modules/features/jobs';
import RunActivityCard from './RunActivityCard.svelte';

/**
 * The three run-activity sentences interpolate a **positional** `{0}`, because
 * the `<Trans>` they were ported from gave lingui an expression rather than an
 * identifier. `dashboard.messages.ts` keeps them that way on purpose: a msgid is
 * the whole string, placeholders included, so naming them would have minted new
 * messages and quietly dropped the French — with `extract` happy,
 * `i18n:collisions` at zero and coverage unchanged.
 *
 * This is the test that notices.
 */
withLocale('fr', messages);

const summary = (overrides: Partial<JobsSummary> = {}): JobsSummary =>
  ({
    total: 10,
    completed: 6,
    failed: 2,
    cancelled: 1,
    orphaned: 1,
    pending: 0,
    running: 0,
    successRate: 0.6,
    lastRunAt: null,
    ...overrides,
  }) as JobsSummary;

describe('RunActivityCard in French', () => {
  it('keeps both numbers of the partial-window sentence', () => {
    render(RunActivityCard, { summary: summary(), totalRuns: 250, truncated: true, loading: false });

    expect(
      screen.getByText(/sur les 10 dernières exécutions parmi 250/),
    ).toBeInTheDocument();
  });

  it('translates the whole-history sentence', () => {
    render(RunActivityCard, { summary: summary(), totalRuns: 10, truncated: false, loading: false });

    expect(screen.getByText(/sur l'ensemble des 10 exécutions/)).toBeInTheDocument();
  });

  it('interpolates the count into the in-flight badge', () => {
    render(RunActivityCard, {
      summary: summary({ pending: 1, running: 4 }),
      totalRuns: 10,
      truncated: false,
      loading: false,
    });

    expect(screen.getByText('5 en cours')).toBeInTheDocument();
  });
});
