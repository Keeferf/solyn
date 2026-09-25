import {
  File,
  HardDrive,
  Info,
  Cpu,
  Download,
  Loader,
  X,
} from "lucide-react";
import { GGUFFile } from "../hooks/useHuggingFaceModels";
import {
  formatFileSize,
  getQuantizationDescription,
  formatParameterCount,
  getQuantizationLabel,
} from "../utils/modalUtils";

interface SelectedFileDetailsProps {
  file: GGUFFile;
  modelId: string;
  isDownloading: boolean;
  isCancelling?: boolean;
  onDownload: (modelId: string, filename: string) => void;
  onCancel?: (modelId: string, filename: string) => void;
}

export const SelectedFileDetails = ({
  file,
  modelId,
  isDownloading,
  isCancelling = false,
  onDownload,
  onCancel,
}: SelectedFileDetailsProps) => {
  const quant = file.quantization || getQuantizationLabel(file.filename);

  const handleDownload = () => {
    onDownload(modelId, file.filename);
  };

  const handleCancel = () => {
    if (onCancel) {
      onCancel(modelId, file.filename);
    }
  };

  const isDisabled = isDownloading || isCancelling;

  return (
    <div className="p-4 bg-black/50 rounded-lg border border-white/10">
      <div className="flex items-center justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <File className="text-purple-accent" size={16} />
            <span className="text-white text-sm font-mono truncate">
              {file.filename}
            </span>
          </div>
          <div className="flex items-center gap-4 mt-2 text-xs text-white/40 flex-wrap">
            <span className="flex items-center gap-1">
              <HardDrive size={12} />
              {formatFileSize(file.size)}
            </span>
            <span className="flex items-center gap-1">
              <Info size={12} />
              {getQuantizationDescription(quant)}
            </span>
            {file.parameter_count && (
              <span className="flex items-center gap-1 text-emerald-400">
                <Cpu size={12} />
                {formatParameterCount(file.parameter_count)}
              </span>
            )}
          </div>
          {isDownloading && !isCancelling && (
            <span className="text-purple-accent text-xs flex items-center gap-1 mt-2">
              <Loader className="animate-spin" size={12} />
              Downloading...
            </span>
          )}
          {isCancelling && (
            <span className="text-yellow-400 text-xs flex items-center gap-1 mt-2">
              <Loader className="animate-spin" size={12} />
              Cancelling...
            </span>
          )}
        </div>
        <div className="ml-4 shrink-0">
          {isDownloading && !isCancelling ? (
            <button
              onClick={handleCancel}
              disabled={isCancelling}
              className="px-6 py-2.5 bg-red-500/20 hover:bg-red-500/30 text-red-400 rounded-lg text-sm font-medium transition-all flex items-center gap-2 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed border border-red-500/20 hover:border-red-500/40"
            >
              <X size={16} />
              Cancel
            </button>
          ) : (
            <button
              onClick={handleDownload}
              disabled={isDisabled}
              className="px-6 py-2.5 bg-purple-accent hover:bg-purple-accent/80 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-all flex items-center gap-2 cursor-pointer disabled:cursor-not-allowed shrink-0"
            >
              {isCancelling ? (
                <>
                  <Loader className="animate-spin" size={16} />
                  Cancelling...
                </>
              ) : isDownloading ? (
                <>
                  <Loader className="animate-spin" size={16} />
                  Downloading...
                </>
              ) : (
                <>
                  <Download size={16} />
                  Download
                </>
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
