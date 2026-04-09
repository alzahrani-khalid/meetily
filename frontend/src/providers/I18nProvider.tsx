'use client';

/**
 * I18nProvider — the SOLE importer of `next-intl` in Meetily.
 *
 * Wraps NextIntlClientProvider per D-07 so that every next-intl coupling
 * lives in exactly one file. Future library swaps or version bumps stay
 * contained here.
 *
 * Phase 2 scope (D-06):
 *   - Client-provider mode ONLY. No `i18n.ts` server-side config, no
 *     `app/[locale]/` route segment, no middleware, no `getTranslations`.
 *   - The provider takes `locale` and `messages` as props from the caller
 *     (`layout.tsx` after the bootstrap detector resolves).
 */

import { NextIntlClientProvider, type AbstractIntlMessages } from 'next-intl';
import type { ReactNode } from 'react';

export type Locale = 'en' | 'ar';
export type Messages = AbstractIntlMessages;

interface I18nProviderProps {
  locale: Locale;
  messages: Messages;
  children: ReactNode;
}

export function I18nProvider({ locale, messages, children }: I18nProviderProps) {
  return (
    <NextIntlClientProvider locale={locale} messages={messages}>
      {children}
    </NextIntlClientProvider>
  );
}

// Re-export useTranslations so Plan 05 components can import from @/providers/I18nProvider
// (preserves D-07: I18nProvider.tsx is the sole file importing from 'next-intl')
export { useTranslations } from 'next-intl';
