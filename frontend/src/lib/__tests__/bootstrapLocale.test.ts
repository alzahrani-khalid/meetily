import { describe, it, expect } from 'vitest';
import { bootstrapLocale } from '@/lib/bootstrapLocale';
import type { UserPreferences } from '@/services/preferencesService';

// Helper to build a UserPreferences fixture with overrides
const prefs = (overrides: Partial<UserPreferences> = {}): UserPreferences => ({
  uiLocale: 'en',
  summaryLanguage: 'en',
  transcriptionLanguage: 'auto',
  bootstrapped: false,
  ...overrides,
});

describe('bootstrapLocale (D-08 pure function)', () => {
  it('T2-01: already-bootstrapped Arabic returns { uiLocale: ar, persist: null }', () => {
    const result = bootstrapLocale(
      prefs({ uiLocale: 'ar', bootstrapped: true }),
      'en-US', // irrelevant — bootstrapped short-circuits
    );
    expect(result).toEqual({ uiLocale: 'ar', persist: null });
  });

  it('T2-02: already-bootstrapped English returns { uiLocale: en, persist: null }', () => {
    const result = bootstrapLocale(
      prefs({ uiLocale: 'en', bootstrapped: true }),
      'ar-SA', // irrelevant — bootstrapped short-circuits
    );
    expect(result).toEqual({ uiLocale: 'en', persist: null });
  });

  it('T2-03: first-run with navigator.language "ar-SA" flips to Arabic + persists both fields', () => {
    const result = bootstrapLocale(
      prefs({ uiLocale: 'en', bootstrapped: false }),
      'ar-SA',
    );
    expect(result).toEqual({
      uiLocale: 'ar',
      persist: { uiLocale: 'ar', bootstrapped: true },
    });
  });

  it('T2-04: first-run with navigator.language "ar" flips to Arabic + persists both fields', () => {
    const result = bootstrapLocale(
      prefs({ uiLocale: 'en', bootstrapped: false }),
      'ar',
    );
    expect(result).toEqual({
      uiLocale: 'ar',
      persist: { uiLocale: 'ar', bootstrapped: true },
    });
  });

  it('T2-05: first-run with navigator.language "en-US" keeps English and persists only bootstrapped flag', () => {
    const result = bootstrapLocale(
      prefs({ uiLocale: 'en', bootstrapped: false }),
      'en-US',
    );
    expect(result).toEqual({
      uiLocale: 'en',
      persist: { bootstrapped: true },
    });
  });

  it('T2-06: first-run with navigator.language undefined keeps English and persists only bootstrapped flag', () => {
    const result = bootstrapLocale(
      prefs({ uiLocale: 'en', bootstrapped: false }),
      undefined,
    );
    expect(result).toEqual({
      uiLocale: 'en',
      persist: { bootstrapped: true },
    });
  });
});
