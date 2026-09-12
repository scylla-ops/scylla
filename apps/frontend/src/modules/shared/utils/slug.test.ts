import { describe, it, expect } from 'vitest';
import { slugifyOrgName } from './slug';

describe('slugifyOrgName', () => {
  it('lowercases and hyphenates spaces', () => {
    expect(slugifyOrgName('My Organization')).toBe('my-organization');
  });

  it('collapses runs of non-alphanumeric characters into a single hyphen', () => {
    expect(slugifyOrgName('Foo___Bar!!  Baz')).toBe('foo-bar-baz');
  });

  it('trims leading/trailing whitespace before slugifying', () => {
    expect(slugifyOrgName('  Padded Name  ')).toBe('padded-name');
  });

  it('strips leading/trailing hyphens produced by punctuation at the edges', () => {
    expect(slugifyOrgName('--Weird--')).toBe('weird');
    expect(slugifyOrgName('!Org!')).toBe('org');
  });

  it('keeps digits', () => {
    expect(slugifyOrgName('Team 42')).toBe('team-42');
  });

  it('returns an empty string for input with no alphanumeric characters', () => {
    expect(slugifyOrgName('!!!')).toBe('');
  });
});
