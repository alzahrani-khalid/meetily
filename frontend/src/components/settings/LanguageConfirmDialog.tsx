'use client';

/**
 * LanguageConfirmDialog — shadcn AlertDialog per UI-SPEC Visual Contract.
 *
 * Responsibilities:
 *   1. Confirm or cancel the locale switch (prevents accidental reload)
 *   2. Block the confirm button while a recording is active (D-16)
 *   3. On confirm: await setUserPreferences({ uiLocale }) then window.location.reload()
 *      Phase 1's atomic transaction handles auto-repoint to localWhisper if flipping to 'ar'
 *
 * Accessibility:
 *   - aria-labelledby via AlertDialogTitle
 *   - aria-describedby via AlertDialogDescription
 *   - Auto-focus primary CTA (shadcn default)
 *   - Esc closes (shadcn default)
 *
 * Zero physical-direction classes. All strings via useTranslations.
 */

import { useState } from 'react';
import { Loader2 } from 'lucide-react';
import { toast } from 'sonner';
import { useTranslations } from '@/providers/I18nProvider';
import { useRecordingState } from '@/contexts/RecordingStateContext';
import { setUserPreferences, type UiLocale } from '@/services/preferencesService';
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogAction,
  AlertDialogCancel,
} from '@/components/ui/alert-dialog';

interface LanguageConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  targetLocale: UiLocale;
  targetLanguageName: string;
}

export function LanguageConfirmDialog({
  open,
  onOpenChange,
  targetLocale,
  targetLanguageName,
}: LanguageConfirmDialogProps) {
  const t = useTranslations('settings.language');
  const tLang = useTranslations('languageConfirm');
  const { isRecording } = useRecordingState();
  const [isPersisting, setIsPersisting] = useState(false);

  const handleConfirm = async () => {
    if (isRecording || isPersisting) return;
    setIsPersisting(true);
    try {
      await setUserPreferences({ uiLocale: targetLocale });
      // Full reload per UI-SPEC §Switch flow step 6. Matches spec §12.4
      // and the existing onboarding-complete reload pattern in layout.tsx.
      window.location.reload();
    } catch (err) {
      console.error('[LanguageConfirmDialog] setUserPreferences failed:', err);
      toast.error(t('error.persistFailed'));
      setIsPersisting(false);
      // Dialog stays open with CTA re-enabled per UI-SPEC Interaction States error row
    }
  };

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle className="text-h1 font-semibold">
            {t('confirm.title', { lang: targetLanguageName })}
          </AlertDialogTitle>
          <AlertDialogDescription className="text-body font-normal">
            {t('confirm.body')}
          </AlertDialogDescription>
          {isRecording && (
            <p className="mt-2 text-small font-normal text-muted-foreground">
              {t('confirm.recordingBlocker')}
            </p>
          )}
          {targetLocale === 'ar' && (
            <p className="mt-2 text-sm font-normal text-muted-foreground italic">
              {tLang('providerRepoint')}
            </p>
          )}
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={isPersisting}>
            {t('confirm.cancelCta')}
          </AlertDialogCancel>
          <AlertDialogAction
            type="button"
            disabled={isRecording || isPersisting}
            onClick={handleConfirm}
            className="h-11 text-body font-semibold"
          >
            {isPersisting && (
              <Loader2 className="me-2 h-4 w-4 animate-spin" aria-hidden="true" />
            )}
            {t('confirm.primaryCta')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
