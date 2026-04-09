'use client';

/**
 * LanguageSwitcher — Settings row per UI-SPEC §Settings Language Switcher Visual Contract.
 *
 * Consumes:
 *   - useTranslations('settings.language') for all copy
 *   - useLocale() for the current uiLocale (source of truth: I18nProvider locale prop)
 *   - LanguageConfirmDialog for the confirm-and-reload flow
 *
 * Visual contract:
 *   - Section heading h2 with text-h1 (24px / 600)
 *   - Description text-small text-muted-foreground font-normal mt-1
 *   - RadioGroup with 2 items, gap-3 between rows, per-row rounded-md border p-4
 *   - Selected: ring-2 ring-primary + border-primary; unselected: border-input
 *   - Hover: bg-muted/50
 *   - Action Button variant default, text-body font-semibold, h-11 touch target
 *   - Button disabled when selection === current locale
 *
 * Zero physical-direction classes. useTranslations is the only source of strings.
 */

import { useState } from 'react';
import { useTranslations, useLocale } from '@/providers/I18nProvider';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { LanguageConfirmDialog } from '@/components/settings/LanguageConfirmDialog';
import type { UiLocale } from '@/services/preferencesService';

export function LanguageSwitcher() {
  const t = useTranslations('settings.language');

  // Get current locale from I18nProvider (which receives it from layout.tsx bootstrap).
  // Deviation from plan: plan used useConfig() which doesn't expose uiLocale.
  // useLocale() (re-exported from I18nProvider per D-07) is the correct source.
  const locale = useLocale();
  const currentLocale: UiLocale = (locale === 'ar' ? 'ar' : 'en');

  const [selected, setSelected] = useState<UiLocale>(currentLocale);
  const [confirmOpen, setConfirmOpen] = useState(false);

  const targetLocale = selected;
  const targetLanguageName = t(`option.${targetLocale}` as 'option.en' | 'option.ar');
  const isSameAsCurrent = selected === currentLocale;

  return (
    <section className="border-t pt-8" aria-labelledby="interface-language-heading">
      <h2
        id="interface-language-heading"
        className="text-h1 font-semibold text-foreground"
      >
        {t('sectionTitle')}
      </h2>
      <p className="mt-1 text-small font-normal text-muted-foreground">
        {t('description')}
      </p>

      <RadioGroup
        value={selected}
        onValueChange={(value) => setSelected(value as UiLocale)}
        className="mt-6 flex flex-col gap-3"
      >
        <div
          className={`flex items-center gap-3 rounded-md border p-4 hover:bg-muted/50 ${
            selected === 'en' ? 'border-primary ring-2 ring-primary' : 'border-input'
          }`}
        >
          <RadioGroupItem value="en" id="interface-language-en" />
          <Label htmlFor="interface-language-en" className="text-body font-normal">
            {t('option.en')}
          </Label>
        </div>
        <div
          className={`flex items-center gap-3 rounded-md border p-4 hover:bg-muted/50 ${
            selected === 'ar' ? 'border-primary ring-2 ring-primary' : 'border-input'
          }`}
        >
          <RadioGroupItem value="ar" id="interface-language-ar" />
          <Label htmlFor="interface-language-ar" className="text-body font-normal">
            {t('option.ar')}
          </Label>
        </div>
      </RadioGroup>

      <div className="mt-6">
        <Button
          type="button"
          variant="default"
          disabled={isSameAsCurrent}
          className="h-11 text-body font-semibold"
          onClick={() => setConfirmOpen(true)}
        >
          {isSameAsCurrent
            ? t('currentLabel')
            : t('switchCta', { lang: targetLanguageName })}
        </Button>
      </div>

      <LanguageConfirmDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        targetLocale={targetLocale}
        targetLanguageName={targetLanguageName}
      />
    </section>
  );
}
