import { describe, it, expect } from 'vitest';
import { getStatusConfig, STATUS_CONFIG } from './status-config';

describe('getStatusConfig', () => {
  it('resolves a known status to its own config', () => {
    expect(getStatusConfig('running')).toBe(STATUS_CONFIG.running);
    expect(getStatusConfig('completed')).toBe(STATUS_CONFIG.completed);
    expect(getStatusConfig('failed')).toBe(STATUS_CONFIG.failed);
    expect(getStatusConfig('skipped')).toBe(STATUS_CONFIG.skipped);
    expect(getStatusConfig('cancelled')).toBe(STATUS_CONFIG.cancelled);
    expect(getStatusConfig('orphaned')).toBe(STATUS_CONFIG.orphaned);
    expect(getStatusConfig('unknown')).toBe(STATUS_CONFIG.unknown);
  });

  it('falls back to "pending" for an unrecognized status string', () => {
    expect(getStatusConfig('not-a-real-status')).toBe(STATUS_CONFIG.pending);
    expect(getStatusConfig('')).toBe(STATUS_CONFIG.pending);
  });

  it('every entry carries a full, non-empty set of style fields (no accidental gaps)', () => {
    for (const [key, config] of Object.entries(STATUS_CONFIG)) {
      expect(config.badgeClassName, `${key}.badgeClassName`).not.toBe('');
      expect(config.iconClassName, `${key}.iconClassName`).not.toBe('');
      expect(config.barClassName, `${key}.barClassName`).not.toBe('');
      expect(config.barHoverClassName, `${key}.barHoverClassName`).not.toBe('');
      expect(config.dotClassName, `${key}.dotClassName`).not.toBe('');
      expect(config.textClassName, `${key}.textClassName`).not.toBe('');
      expect(config.icon).toBeDefined();
      expect(config.label.id).toBeTruthy();
    }
  });
});
