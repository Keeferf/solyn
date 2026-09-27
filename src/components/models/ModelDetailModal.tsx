import { useState, useEffect } from "react";
import { Folder, X } from "lucide-react";
import { GGUFFile } from "./hooks/useHuggingFaceModels";
import { useModelDetails } from "./hooks/useModelDetails";
import {
  getValidFiles,
  groupFilesByQuantization,
  getDefaultSelectedFile,
} from "./utils/modalUtils";
import { LoadingState } from "./modal-details/LoadingState";
import { ErrorState } from "./modal-details/ErrorState";
import { ModelInfo } from "./modal-details/ModelInfo";
import { QuantizationCard } from "./modal-details/QuantizationCard";
import { SelectedFileDetails } from "./modal-details/SelectedFileDetails";
import { invoke } from "@tauri-apps/api/core";

interface ModelDetailModalProps {
  modelId: string | null;
  isOpen: boolean;
  onClose: () => void;
  onDownload: (modelId: string, filename: string) => Promise<void>;
  downloadingModels: Set<string>;
  isDownloading?: (modelId: string, filename: string) => boolean;
  onCancelDownload?: (modelId: string, filename: string) => void;
}

export const ModelDetailModal = ({
  modelId,
  isOpen,
  onClose,
  onDownload,
  downloadingModels,
  isDownloading = (modelId: string, filename: string) => {
    return downloadingModels.has(`${modelId}::${filename}`);
  },
  onCancelDownload,
}: ModelDetailModalProps) => {
  const { details, isLoading, error } = useModelDetails(modelId, isOpen);
  const [selectedFile, setSelectedFile] = useState<GGUFFile | null>(null);
  const [cancellingFile, setCancellingFile] = useState<string | null>(null);

  useEffect(() => {
    if (details && details.gguf_files.length > 0) {
      const defaultFile = getDefaultSelectedFile(details.gguf_files);
      setSelectedFile(defaultFile);
    } else {
      setSelectedFile(null);
    }
  }, [details]);

  useEffect(() => {
    if (!isOpen) {
      setCancellingFile(null);
    }
  }, [isOpen]);

  const handleCancelDownload = async (modelId: string, filename: string) => {
    setCancellingFile(filename);
    try {
      await invoke("cancel_huggingface_download", {
        modelId,
        filename,
      });
      if (onCancelDownload) {
        onCancelDownload(modelId, filename);
      }
    } catch (error) {
      setCancellingFile(null);
    }
  };

  const isFileDownloading = (filename: string): boolean => {
    if (!modelId) return false;
    return isDownloading(modelId, filename);
  };

  const isFileCancelling = (filename: string): boolean => {
    return cancellingFile === filename;
  };

  if (isLoading) {
    return <LoadingState onClose={onClose} />;
  }

  if (error) {
    return <ErrorState error={error} onClose={onClose} />;
  }

  if (!details || !modelId) return null;

  const validFiles = getValidFiles(details.gguf_files);
  const groupedFiles = groupFilesByQuantization(validFiles);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-fadeIn"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="bg-[#1a1a1a] border border-white/10 rounded-2xl max-w-3xl w-full max-h-[90vh] overflow-hidden shadow-2xl animate-slideUp">
        <div className="flex items-start justify-between p-6 border-b border-white/10">
          <ModelInfo details={details} />
          <button
            onClick={onClose}
            className="p-2 hover:bg-white/10 rounded-lg transition-all text-white/60 hover:text-white cursor-pointer"
          >
            <X size={20} />
          </button>
        </div>

        <div className="p-6 overflow-y-auto max-h-[calc(90vh-180px)]">
          {details.description && (
            <div className="mb-6 p-4 bg-black rounded-lg border border-white/5">
              <p className="text-white/70 text-sm leading-relaxed">
                {details.description}
              </p>
            </div>
          )}

          <div className="mb-6">
            <h4 className="text-sm font-semibold text-white/60 mb-3 flex items-center gap-2">
              <Folder size={16} />
              Available Quantizations ({validFiles.length} files)
            </h4>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {Object.entries(groupedFiles).map(([quant, files]) => {
                const isSelected =
                  selectedFile &&
                  files.some((f) => f.filename === selectedFile.filename);
                return (
                  <QuantizationCard
                    key={quant}
                    quant={quant}
                    files={files}
                    isSelected={!!isSelected}
                    onSelect={setSelectedFile}
                  />
                );
              })}
            </div>

            {validFiles.length === 0 && (
              <div className="text-center py-8 text-white/40">
                <p>No recognized quantization formats found</p>
                <p className="text-xs mt-1">
                  This model may have no GGUF files or uses an unsupported
                  naming convention
                </p>
              </div>
            )}
          </div>

          {selectedFile && (
            <SelectedFileDetails
              file={selectedFile}
              modelId={modelId}
              isDownloading={isFileDownloading(selectedFile.filename)}
              isCancelling={isFileCancelling(selectedFile.filename)}
              onDownload={onDownload}
              onCancel={handleCancelDownload}
            />
          )}
        </div>

        <div className="p-4 border-t border-white/10 flex justify-between items-center">
          <span className="text-xs text-white/30">
            {validFiles.length} GGUF file
            {validFiles.length > 1 ? "s" : ""} available
          </span>
          <button
            onClick={onClose}
            className="px-4 py-2 bg-black hover:bg-white/10 rounded-lg text-white/60 hover:text-white transition-all text-sm cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
