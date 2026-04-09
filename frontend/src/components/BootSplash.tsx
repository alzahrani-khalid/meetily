'use client';

import { Loader2 } from 'lucide-react';

/**
 * BootSplash — blocking loader per UI-SPEC BootSplash Visual Contract.
 *
 * Rendered by RootLayout when `initialPreferences` is null (during the
 * preference bootstrap useEffect). Dismounts once the I18nProvider can
 * mount with resolved locale.
 *
 * Visual contract (UI-SPEC):
 *   - Full-bleed min-h-screen, flex centered
 *   - Wordmark "Meetily" (text-display 32px, font-semibold 600 — overriding
 *     the token's default 700 to honor Phase 2's 2-weight contract)
 *   - gap-4 (16px)
 *   - Lucide Loader2 spinner (w-6 h-6, text-primary, animate-spin)
 *   - gap-2 (8px) absorbed into the flex gap-4 pattern
 *   - Tagline (text-small 14px, text-muted-foreground, font-normal 400)
 *
 * Accessibility:
 *   - role="status" on container
 *   - aria-live="polite" on the container
 *   - visually-hidden "Loading Meetily" on the spinner (sr-only span)
 *
 * Direction: fixed LTR during splash (we don't know the locale yet).
 * text-center is RTL-safe.
 *
 * Language: BootSplash renders BEFORE I18nProvider mounts, so useTranslations
 * would throw at runtime. Strings are inlined to match the English catalogue.
 * UI-SPEC §Specific Ideas explicitly accepts the brief English flash on an
 * Arabic first-run — the next paint (with I18nProvider mounted) is Arabic.
 */
export function BootSplash() {
  return (
    <div
      role="status"
      aria-live="polite"
      className="flex min-h-screen flex-col items-center justify-center gap-4 bg-background text-center"
      dir="ltr"
    >
      <h1 className="text-display font-semibold text-foreground">Meetily</h1>
      <Loader2 className="h-6 w-6 animate-spin text-primary" aria-hidden="true" />
      <span className="sr-only">Loading Meetily</span>
      <p className="text-small font-normal text-muted-foreground">
        Preparing your meeting assistant…
      </p>
    </div>
  );
}
