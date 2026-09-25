import { Download, ExternalLink } from "lucide-react";

interface InstallInfo {
  platform: string;
  command: string;
  estimated_time: string;
}

interface DownloadDetailsProps {
  installInfo: InstallInfo;
  platformDisplay: string;
  isOllamaInstalled: boolean | null;
  onDownload: () => void;
  isDownloading: boolean;
}

export const DownloadDetails = ({
  installInfo,
  platformDisplay,
  isOllamaInstalled,
  onDownload,
  isDownloading,
}: DownloadDetailsProps) => {
  return (
    <div className="space-y-6">
      <div className="bg-white/5 border border-white/10 rounded-xl p-6 text-left">
        <h3 className="text-sm font-semibold text-white/80 mb-3">
          Download Details
        </h3>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between text-white/60 gap-4">
            <span className="shrink-0">Platform</span>
            <span className="text-white text-right">{platformDisplay}</span>
          </div>
          <div className="flex justify-between text-white/60 gap-4">
            <span className="shrink-0">Command</span>
            <span className="text-white text-right font-mono text-xs break-all">
              {installInfo.command}
            </span>
          </div>
          <div className="flex justify-between text-white/60 gap-4">
            <span className="shrink-0">Estimated Time</span>
            <span className="text-white text-right">
              {installInfo.estimated_time}
            </span>
          </div>
        </div>
      </div>

      <button
        onClick={onDownload}
        disabled={isDownloading}
        className="px-8 py-3 bg-purple-accent hover:bg-purple-accent/80 disabled:opacity-50 text-white rounded-xl font-medium transition-all flex items-center justify-center gap-2 mx-auto cursor-pointer"
      >
        <Download size={18} />
        {isOllamaInstalled
          ? "Reinstall Ollama"
          : `Download Ollama for ${platformDisplay}`}
      </button>

      <div className="text-xs text-white/30 text-center">
        <span className="block">Or manually download from </span>
        <a
          href="https://ollama.com/download"
          target="_blank"
          rel="noopener noreferrer"
          className="text-purple-accent hover:underline inline-flex items-center gap-1"
        >
          ollama.com/download
          <ExternalLink size={12} />
        </a>
      </div>
    </div>
  );
};
