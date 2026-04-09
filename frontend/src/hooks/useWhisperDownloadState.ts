import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { WhisperAPI } from '@/lib/whisper';
import type { ModelInfo } from '@/lib/whisper';

interface WhisperDownloadState {
  isReady: boolean;
  isDownloading: boolean;
  progress: number;
  error: string | null;
}

export function useWhisperDownloadState(modelName: string): WhisperDownloadState & { startDownload: () => Promise<void>; retryDownload: () => Promise<void> } {
  const [isReady, setIsReady] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  // Check initial model state on mount (survives page reload)
  useEffect(() => {
    WhisperAPI.getAvailableModels().then((models: ModelInfo[]) => {
      const model = models.find((m) => m.name === modelName);
      if (model) {
        if (model.status === 'Available') {
          setIsReady(true);
        } else if (typeof model.status === 'object' && model.status !== null && 'Downloading' in model.status) {
          setIsDownloading(true);
          setProgress(model.status.Downloading);
        }
      }
    }).catch((err: unknown) => console.error('[useWhisperDownloadState] Failed to check models:', err));
  }, [modelName]);

  // Listen for download events
  useEffect(() => {
    const unlistenProgress = listen<{ modelName: string; progress: number }>('model-download-progress', (event) => {
      if (event.payload.modelName === modelName) {
        setIsDownloading(true);
        setError(null);
        setProgress(Math.round(event.payload.progress));
      }
    });

    const unlistenComplete = listen<{ modelName: string }>('model-download-complete', (event) => {
      if (event.payload.modelName === modelName) {
        setIsReady(true);
        setIsDownloading(false);
        setProgress(100);
        setError(null);
      }
    });

    const unlistenError = listen<{ modelName: string; error?: string }>('model-download-error', (event) => {
      if (event.payload.modelName === modelName) {
        setIsDownloading(false);
        setError(event.payload.error || 'Download failed');
      }
    });

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenComplete.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, [modelName]);

  const startDownload = useCallback(async () => {
    try {
      setIsDownloading(true);
      setError(null);
      setProgress(0);
      await WhisperAPI.downloadModel(modelName);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Download failed');
      setIsDownloading(false);
    }
  }, [modelName]);

  const retryDownload = useCallback(async () => {
    setError(null);
    await startDownload();
  }, [startDownload]);

  return { isReady, isDownloading, progress, error, startDownload, retryDownload };
}
