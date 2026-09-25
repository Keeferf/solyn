import { X, Loader, CircleCheck, CircleAlert } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface DownloadStatusDisplayProps {
  modelId: string;
  filename: string;
  progress: number;
  message: string;
  status: string;
  onCancel?: () => void;
}

export const DownloadStatusDisplay = ({
  modelId,
  filename,
  progress,
  status,
  message,
  onCancel,
}: DownloadStatusDisplayProps) => {
  const displayName =
    filename.length > 40 ? filename.substring(0, 37) + "..." : filename;

  const isComplete = status === "complete";
  const isError = status === "error";
  const isCancelled = status === "cancelled";
  const isActive = status === "downloading" || status === "starting";
  const isStarting = status === "starting";
  const isProcessing =
    status === "processing" ||
    status === "generating_modelfile" ||
    status === "creating_ollama_model" ||
    status === "finalizing";
  const isOllamaCreation = status === "creating_ollama_model";

  const progressColor = isError
    ? "bg-error"
    : isComplete
      ? "bg-success"
      : isCancelled
        ? "bg-warning"
        : isProcessing
          ? "bg-info animate-pulse"
          : "bg-purple-accent";

  const progressValue = Math.min(Math.max(progress, 0), 100);

  const handleCancel = async () => {
    try {
      await invoke("cancel_huggingface_download", {
        modelId,
        filename,
      });
      if (onCancel) {
        onCancel();
      }
    } catch {}
  };

  const StatusIcon = () => {
    if (isComplete)
      return <CircleCheck className="text-success shrink-0" size={16} />;
    if (isError)
      return <CircleAlert className="text-error shrink-0" size={16} />;
    if (isCancelled)
      return <CircleAlert className="text-warning shrink-0" size={16} />;
    if (isProcessing)
      return <Loader className="animate-spin text-info shrink-0" size={16} />;
    if (isStarting || isActive)
      return (
        <Loader
          className="animate-spin text-purple-accent shrink-0"
          size={16}
        />
      );
    return null;
  };

  const showCancelButton = isActive && !isComplete && !isError && !isCancelled;
  const isIndeterminate = isProcessing && !isOllamaCreation;

  return (
    <div className="bg-black/50 rounded-lg border border-white/10 p-4">
      <div className="flex items-center justify-between mb-1">
        <div className="flex-1 min-w-0 flex items-center gap-2">
          <StatusIcon />
          <span className="font-inter text-white text-base truncate">
            {displayName}
          </span>
        </div>
        <div className="flex items-center gap-3 ml-4 shrink-0">
          {showCancelButton && (
            <button
              onClick={handleCancel}
              className="px-3 py-1 bg-error-bg hover:bg-error-border text-error rounded-lg text-xs transition-all cursor-pointer border border-error-border hover:border-error-border flex items-center gap-1"
            >
              <X size={12} />
              Cancel
            </button>
          )}
        </div>
      </div>

      {message && (
        <div className="ml-6 mb-2">
          <span className="font-inter text-white/40 text-xs">{message}</span>
        </div>
      )}

      <div className="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
        <div
          className={`h-full transition-all duration-300 ${progressColor} ${isIndeterminate ? "w-full animate-pulse" : ""}`}
          style={{
            width: isComplete
              ? "100%"
              : isError
                ? "100%"
                : isCancelled
                  ? "100%"
                  : isIndeterminate
                    ? "100%"
                    : `${Math.min(progressValue, 100)}%`,
          }}
        />
      </div>
    </div>
  );
};
