/**
 * User preferences service — TypeScript wrapper around the Tauri
 * `get_user_preferences` / `set_user_preferences` commands defined in
 * `frontend/src-tauri/src/preferences/commands.rs`.
 *
 * Replaces the legacy 4-way preferences desync (React state / localStorage /
 * Rust process-global / SQLite `settings` table) per PREFS-01..PREFS-04.
 */

import { invoke } from '@tauri-apps/api/core';

export type UiLocale = 'en' | 'ar';
export type SummaryLanguage = 'en' | 'ar';

export interface UserPreferences {
  uiLocale: UiLocale;
  summaryLanguage: SummaryLanguage;
  transcriptionLanguage: string; // 'auto' | 'en' | 'ar' | ISO code
}

/**
 * The server-side patch shape includes an optional `provider` field used by
 * Phase 4 (TRANS-02) to explicitly set the transcript provider. Phase 1
 * includes it because the Rust `UserPreferencesPatch` has it (A1 Option B),
 * and the reject branch enforces `provider === 'parakeet'` cannot coexist
 * with `uiLocale === 'ar'`.
 */
export interface UserPreferencesPatch extends Partial<UserPreferences> {
  provider?: string;
}

export async function getUserPreferences(): Promise<UserPreferences> {
  return await invoke<UserPreferences>('get_user_preferences');
}

export async function setUserPreferences(
  patch: UserPreferencesPatch,
): Promise<UserPreferences> {
  return await invoke<UserPreferences>('set_user_preferences', { patch });
}
