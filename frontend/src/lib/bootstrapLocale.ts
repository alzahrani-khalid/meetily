/**
 * First-run locale detection helper — D-08 pure function.
 *
 * Called once from the RootLayout `useEffect` in `layout.tsx`. Given the
 * persisted `UserPreferences` and the current `navigator.language`, decides
 * (a) which locale to show and (b) which fields, if any, to persist via
 * Phase 1's atomic `setUserPreferences` call.
 *
 * This function is intentionally pure:
 *   - zero I/O
 *   - zero reads of globals (window, navigator, or browser storage APIs)
 *   - zero Tauri command invocations
 * The caller MUST supply `navigatorLanguage` (typically `navigator.language`).
 *
 * Decision matrix (CONTEXT.md D-08):
 *
 *   bootstrapped  | navigator | result
 *   --------------+-----------+---------------------------------------------
 *   true (any)    | *         | { uiLocale: prefs.uiLocale, persist: null }
 *   false         | ar*       | { uiLocale: 'ar', persist: { uiLocale: 'ar', bootstrapped: true } }
 *   false         | other/undef | { uiLocale: prefs.uiLocale, persist: { bootstrapped: true } }
 *
 * The "other" branch preserves the existing uiLocale (typically 'en' on
 * first run from Phase 1's seed) and only records that detection has run.
 */

import type {
  UserPreferences,
  UserPreferencesPatch,
} from '@/services/preferencesService';

export interface BootstrapResult {
  uiLocale: 'en' | 'ar';
  persist: Partial<UserPreferencesPatch> | null;
}

export function bootstrapLocale(
  prefs: UserPreferences,
  navigatorLanguage: string | undefined,
): BootstrapResult {
  // Branch 1: already bootstrapped — never re-detect. (T2-01, T2-02)
  if (prefs.bootstrapped === true) {
    return {
      uiLocale: prefs.uiLocale,
      persist: null,
    };
  }

  // Branch 2: first run, navigator reports Arabic. (T2-03, T2-04)
  if (
    typeof navigatorLanguage === 'string' &&
    navigatorLanguage.toLowerCase().startsWith('ar')
  ) {
    return {
      uiLocale: 'ar',
      persist: { uiLocale: 'ar', bootstrapped: true },
    };
  }

  // Branch 3: first run, navigator is non-Arabic or undefined. (T2-05, T2-06)
  // Keep whatever the row currently has (Phase 1 seeds 'en'), only mark
  // detection as complete so we never re-run.
  return {
    uiLocale: prefs.uiLocale as 'en' | 'ar',
    persist: { bootstrapped: true },
  };
}
