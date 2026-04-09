'use client';

import { useEffect } from 'react';
import { useTranslations } from '@/providers/I18nProvider';
import { Progress } from '@/components/ui/progress';
import { useWhisperDownloadState } from '@/hooks/useWhisperDownloadState';

interface WhisperDownloadGateProps {
  modelName: string;
  onReady?: () => void;
  children: React.ReactNode;
}

export function WhisperDownloadGate({ modelName, onReady, children }: WhisperDownloadGateProps) {
  const { isReady, isDownloading, progress, error, startDownload, retryDownload } = useWhisperDownloadState(modelName);
  const t = useTranslations('recording');

  // Auto-start download if model not ready and not downloading
  useEffect(() => {
    if (!isReady && !isDownloading && !error) {
      startDownload();
    }
  }, [isReady, isDownloading, error, startDownload]);

  // Notify parent when ready
  useEffect(() => {
    if (isReady && onReady) {
      onReady();
    }
  }, [isReady, onReady]);

  if (isReady) {
    return <>{children}</>;
  }

  if (isDownloading) {
    return (
      <div className="flex flex-col items-center gap-2">
        <button
          disabled
          className="min-h-11 min-w-11 rounded-full bg-muted text-muted-foreground px-6 py-3 opacity-70 cursor-not-allowed"
          aria-disabled="true"
          aria-describedby="download-status"
        />
        <Progress value={progress} className="h-2 w-48" />
        <p id="download-status" className="text-sm text-muted-foreground text-center">
          {t('downloadingModel', { progress })}
        </p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center gap-2">
        <button
          disabled
          className="min-h-11 min-w-11 rounded-full bg-muted text-muted-foreground px-6 py-3 opacity-70 cursor-not-allowed"
          aria-disabled="true"
        />
        <button
          onClick={retryDownload}
          className="text-sm text-muted-foreground underline hover:text-foreground"
        >
          {t('modelDownloadError')}
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-2">
      <button
        disabled
        className="min-h-11 min-w-11 rounded-full bg-muted text-muted-foreground px-6 py-3 opacity-70 cursor-not-allowed"
        aria-disabled="true"
      />
      <p className="text-sm text-muted-foreground text-center">
        {t('noArabicModel')}
      </p>
    </div>
  );
}
