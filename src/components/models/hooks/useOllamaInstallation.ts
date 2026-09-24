import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

type DownloadStatus = "Idle" | "Downloading" | "Completed" | "Error";

interface DownloadProgress {
  status: DownloadStatus;
  progress: number;
  message: string;
  log?: string;
}

interface InstallInfo {
  platform: string;
  command: string;
  estimated_time: string;
}

export interface TerminalOutput {
  line: string;
  stream: string;
}

export const useOllamaInstallation = (
  refreshOllamaStatus: () => Promise<void>,
) => {
  const [installInfo, setInstallInfo] = useState<InstallInfo | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress>({
    status: "Idle",
    progress: 0,
    message: "",
  });
  const [isDownloading, setIsDownloading] = useState(false);
  const [terminalLines, setTerminalLines] = useState<TerminalOutput[]>([]);
  const [isTerminalExpanded, setIsTerminalExpanded] = useState(true);
  const terminalEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const fetchInstallInfo = async () => {
      try {
        const info = await invoke<InstallInfo>("get_install_info");
        setInstallInfo(info);
      } catch {}
    };
    fetchInstallInfo();
  }, []);

  useEffect(() => {
    const setupListeners = async () => {
      const unlistenProgress = await listen<DownloadProgress>(
        "download-progress",
        handleDownloadProgress,
      );

      const unlistenTerminal = await listen<TerminalOutput>(
        "terminal-output",
        handleTerminalOutput,
      );

      return () => {
        unlistenProgress();
        unlistenTerminal();
      };
    };

    const unlistenPromise = setupListeners();
    return () => {
      unlistenPromise.then((fn) => {
        if (fn) fn();
      });
    };
  }, [refreshOllamaStatus]);

  const handleDownloadProgress = (event: { payload: DownloadProgress }) => {
    const payload = event.payload;
    setDownloadProgress(payload);

    if (payload.status === "Completed") {
      setIsDownloading(false);
      handleInstallationComplete(refreshOllamaStatus);
    }

    if (payload.status === "Error") {
      setIsDownloading(false);
    }
  };

  const handleInstallationComplete = (
    refreshOllamaStatus: () => Promise<void>,
  ) => {
    let attempts = 0;
    const maxAttempts = 8;

    const tryRefresh = async () => {
      attempts++;
      await refreshOllamaStatus();
      if (attempts < maxAttempts) {
        setTimeout(tryRefresh, 2000);
      }
    };

    setTimeout(tryRefresh, 2000);
  };

  const handleTerminalOutput = (event: { payload: TerminalOutput }) => {
    const line = event.payload.line;

    if (shouldFilterLine(line)) {
      return;
    }

    const isProgressLine = isProgressIndicator(line);

    setTerminalLines((prev) => {
      if (isProgressLine) {
        const lastIndex = prev.length - 1;
        if (lastIndex >= 0) {
          const lastLine = prev[lastIndex].line;
          if (isProgressIndicator(lastLine)) {
            return [...prev.slice(0, lastIndex), event.payload];
          }
        }
        return [...prev, event.payload];
      }
      return [...prev, event.payload];
    });
  };

  const shouldFilterLine = (line: string): boolean => {
    const filteredMessages = [
      "Install complete. Run 'ollama' from the command line.",
      "Run 'ollama' from the command line.",
      "Install complete.",
    ];
    return filteredMessages.some((msg) => line.includes(msg));
  };

  const isProgressIndicator = (line: string): boolean => {
    return (
      line.includes("#") ||
      line.includes("=") ||
      line.includes("█") ||
      line.includes("▓") ||
      line.includes("░") ||
      line.includes("%")
    );
  };

  const handleDownloadOllama = async () => {
    setIsDownloading(true);
    setTerminalLines([]);
    setDownloadProgress({
      status: "Downloading",
      progress: 0,
      message: "Starting download...",
    });
    setIsTerminalExpanded(true);

    try {
      await invoke("download_ollama");
    } catch (error) {
      setDownloadProgress({
        status: "Error",
        progress: 0,
        message: "Download failed",
        log: error instanceof Error ? error.message : "Unknown error occurred",
      });
      setIsDownloading(false);
    }
  };

  const resetState = () => {
    setDownloadProgress({
      status: "Idle",
      progress: 0,
      message: "",
    });
    setTerminalLines([]);
    setIsTerminalExpanded(true);
  };

  useEffect(() => {
    if (terminalEndRef.current) {
      terminalEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [terminalLines]);

  return {
    installInfo,
    downloadProgress,
    isDownloading,
    terminalLines,
    isTerminalExpanded,
    terminalEndRef: terminalEndRef as React.RefObject<HTMLDivElement>,
    handleDownloadOllama,
    resetState,
    setIsTerminalExpanded,
    getPlatformDisplay: () => getPlatformDisplay(installInfo),
  };
};

const getPlatformDisplay = (installInfo: InstallInfo | null): string => {
  if (!installInfo) return "Your Platform";
  const platformMap: Record<string, string> = {
    windows: "Windows",
    linux: "Linux",
  };
  return platformMap[installInfo.platform] || installInfo.platform;
};
