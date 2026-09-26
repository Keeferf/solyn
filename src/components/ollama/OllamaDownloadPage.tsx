import { useEffect } from "react";
import { Download, ExternalLink, CircleX } from "lucide-react";
import { TerminalDisplay } from "@/components/models/TerminalDisplay";
import { useOllamaInstallation } from "@/components/models/hooks/useOllamaInstallation";

export enum DownloadStatus {
  Idle = "Idle",
  Downloading = "Downloading",
  Complete = "Complete",
  Error = "Error",
}

interface OllamaDownloadPageProps {
  onBack?: () => void;
  refreshOllamaStatus: () => Promise<void>;
  isOllamaInstalled: boolean | null;
  onInstallComplete?: () => void;
}

export const OllamaDownloadPage = ({
  onBack,
  refreshOllamaStatus,
  onInstallComplete,
}: OllamaDownloadPageProps) => {
  const {
    installInfo,
    downloadProgress,
    isDownloading,
    terminalLines,
    isTerminalExpanded,
    terminalEndRef,
    handleDownloadOllama,
    setIsTerminalExpanded,
  } = useOllamaInstallation(refreshOllamaStatus);

  const isComplete = downloadProgress.status === ("Complete" as DownloadStatus);
  const isError = downloadProgress.status === ("Error" as DownloadStatus);

  useEffect(() => {
    if (
      installInfo &&
      !isDownloading &&
      downloadProgress.status === ("Idle" as DownloadStatus)
    ) {
      const timer = setTimeout(() => {
        handleDownloadOllama();
      }, 500);
      return () => clearTimeout(timer);
    }
  }, [
    installInfo,
    isDownloading,
    downloadProgress.status,
    handleDownloadOllama,
  ]);

  useEffect(() => {
    if (isComplete) {
      refreshOllamaStatus();

      const timer = setTimeout(() => {
        if (onInstallComplete) {
          onInstallComplete();
        }
        if (onBack) {
          onBack();
        }
      }, 1000);

      return () => clearTimeout(timer);
    }
  }, [isComplete, refreshOllamaStatus, onInstallComplete, onBack]);

  return (
    <div className="flex flex-col h-screen bg-black p-6 overflow-hidden">
      <div className="flex items-center justify-between mb-6 shrink-0">
        <h1 className="text-xl font-semibold text-white">
          {isError ? "Installation Failed" : "Installing Ollama..."}
        </h1>
        <div className="text-sm text-white/40">
          {isError ? (
            <span className="text-red-400">Error occurred</span>
          ) : (
            <span className="text-purple-accent">Please wait...</span>
          )}
        </div>
      </div>

      {isError && (
        <div className="mb-4 p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-red-400 text-center shrink-0">
          <div className="flex items-center justify-center gap-2 mb-1">
            <CircleX size={18} />
            <p className="font-medium">Installation failed</p>
          </div>
          <p className="text-sm text-red-400/60">
            {downloadProgress.message ||
              "Please check your internet connection and try again."}
          </p>
        </div>
      )}

      <div className="flex-1 min-h-0">
        <TerminalDisplay
          terminalLines={terminalLines}
          isTerminalExpanded={isTerminalExpanded}
          onToggleExpand={() => setIsTerminalExpanded(!isTerminalExpanded)}
          terminalEndRef={terminalEndRef}
        />
      </div>

      {isError && (
        <div className="mt-6 flex flex-col sm:flex-row items-center justify-center gap-4 shrink-0">
          <button
            onClick={handleDownloadOllama}
            className="px-8 py-3 bg-purple-accent hover:bg-purple-accent/80 text-white rounded-xl font-medium transition-all flex items-center gap-2 cursor-pointer"
          >
            <Download size={18} />
            Retry Installation
          </button>
          <a
            href="https://ollama.com/download"
            target="_blank"
            rel="noopener noreferrer"
            className="text-purple-accent hover:underline inline-flex items-center gap-1 text-sm"
          >
            Manual Download
            <ExternalLink size={14} />
          </a>
        </div>
      )}
    </div>
  );
};
