import { useState } from "react";
import {
  Trash,
  HardDrive,
  Folder,
  Search,
  X,
  RefreshCw,
  CircleAlert,
} from "lucide-react";
import { useInstalledModels, InstalledModel } from "./hooks/useInstalledModels";

interface InstalledModelsProps {
  onModelClick?: (model: InstalledModel) => void;
}

const formatFileSize = (size: number): string => {
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;

  if (size >= GB) {
    return `${(size / GB).toFixed(2)} GB`;
  }
  if (size >= MB) {
    return `${(size / MB).toFixed(2)} MB`;
  }
  if (size >= KB) {
    return `${(size / KB).toFixed(2)} KB`;
  }
  return `${size} B`;
};

const groupFilesByQuant = (files: InstalledModel["files"]) => {
  const groups: Record<string, InstalledModel["files"]> = {};
  files.forEach((file) => {
    const key = file.quantization || "unquantized";
    if (!groups[key]) groups[key] = [];
    groups[key].push(file);
  });
  return groups;
};

export const InstalledModels = ({ onModelClick }: InstalledModelsProps) => {
  const {
    models,
    loading,
    error,
    searchQuery,
    setSearchQuery,
    deleteModel,
    deleteQuantization,
    refresh,
  } = useInstalledModels();

  const [deleting, setDeleting] = useState<string | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(
    null,
  );
  const [deletingQuant, setDeletingQuant] = useState<{
    modelId: string;
    quant: string;
  } | null>(null);

  const handleDelete = async (modelId: string) => {
    setDeleting(modelId);
    const success = await deleteModel(modelId);
    setDeleting(null);
    setShowDeleteConfirm(null);
    if (!success) {
    }
  };

  const handleDeleteQuant = async (modelId: string, quant: string) => {
    setDeletingQuant({ modelId, quant });
    const success = await deleteQuantization(modelId, quant);
    setDeletingQuant(null);
    if (!success) {
    }
  };

  const handleClearSearch = () => {
    setSearchQuery("");
  };

  if (loading) {
    return (
      <div className="w-full space-y-6">
        <div className="flex items-center justify-between text-sm text-white/40 px-2 py-2 bg-black/50 rounded-lg border border-white/5">
          <div className="flex items-center gap-3">
            <HardDrive className="text-purple-accent" size={16} />
            <span>Loading installed models...</span>
          </div>
        </div>
        <div className="flex items-center justify-center py-16">
          <div className="animate-spin text-purple-accent">
            <RefreshCw size={32} />
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full space-y-6">
        <div className="flex items-center justify-between text-sm text-white/40 px-2 py-2 bg-black/50 rounded-lg border border-white/5">
          <div className="flex items-center gap-3">
            <CircleAlert className="text-error" size={16} />
            <span>Error loading installed models</span>
          </div>
        </div>
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <p className="text-white text-lg mb-2">Failed to load models</p>
          <p className="text-white/40 text-sm mb-4">{error}</p>
          <button
            onClick={refresh}
            className="px-4 py-2 bg-black hover:bg-white/10 rounded-lg text-white transition-all flex items-center gap-2 cursor-pointer"
          >
            <RefreshCw size={16} />
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="w-full">
      <div className="mb-6">
        <div className="relative">
          <Search
            className="absolute left-3 top-1/2 -translate-y-1/2 text-white/30"
            size={16}
          />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search installed models..."
            className="w-full bg-black/50 border border-white/10 rounded-lg px-10 py-2.5 text-white text-sm placeholder:text-white/30 focus:outline-none focus:border-purple-accent focus:ring-2 focus:ring-purple-accent transition-all"
          />
          {searchQuery && (
            <button
              onClick={handleClearSearch}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-white/30 hover:text-white/60 transition-colors cursor-pointer"
            >
              <X size={16} />
            </button>
          )}
        </div>
      </div>

      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2 text-sm text-white/40">
          <HardDrive size={14} />
          <span>
            {models.length} model{models.length !== 1 ? "s" : ""} installed
            {models.length > 0 &&
              ` • ${formatFileSize(models.reduce((sum, m) => sum + m.total_size, 0))} total`}
          </span>
        </div>
        <button
          onClick={refresh}
          className="text-white/40 hover:text-white/60 transition-colors p-2 rounded-lg hover:bg-white/5 cursor-pointer"
          title="Refresh"
        >
          <RefreshCw size={16} />
        </button>
      </div>

      {models.length === 0 ? (
        <div className="text-center py-16">
          {searchQuery ? (
            <>
              <p className="text-white/40 text-lg">No installed models found</p>
              <p className="text-white/30 text-sm mt-2">
                No models match "{searchQuery}"
              </p>
              <button
                onClick={handleClearSearch}
                className="mt-4 px-4 py-2 bg-black hover:bg-white/10 rounded-lg text-white transition-all flex items-center gap-2 mx-auto cursor-pointer"
              >
                <span>Clear search</span>
              </button>
            </>
          ) : (
            <>
              <p className="text-white/40 text-lg">No models installed yet</p>
              <p className="text-white/30 text-sm mt-2">
                Download models from the Browse tab
              </p>
            </>
          )}
        </div>
      ) : (
        <div className="space-y-4">
          {models.map((model) => {
            const quantGroups = groupFilesByQuant(model.files);
            const hasMultipleQuants = Object.keys(quantGroups).length > 1;

            return (
              <div
                key={model.model_id}
                className="bg-black/50 border border-white/10 rounded-xl p-5 transition-all hover:border-white/20"
              >
                <div className="flex items-start justify-between">
                  <div
                    className="flex-1 min-w-0 cursor-pointer"
                    onClick={() => onModelClick?.(model)}
                  >
                    <div className="flex items-center gap-3">
                      <Folder className="text-purple-accent" size={20} />
                      <div>
                        <h4 className="text-white font-semibold text-base">
                          {model.name || model.model_id}
                        </h4>
                        <div className="flex items-center gap-3 mt-1 flex-wrap">
                          <span className="text-white/40 text-xs">
                            {model.author || "Unknown"}
                          </span>
                          <span className="text-white/20 text-xs">•</span>
                          <span className="text-white/30 text-xs flex items-center gap-1">
                            <HardDrive size={12} />
                            {formatFileSize(model.total_size)}
                          </span>
                          {hasMultipleQuants && (
                            <>
                              <span className="text-white/20 text-xs">•</span>
                              <span className="text-purple-accent/60 text-xs">
                                {Object.keys(quantGroups).length} quantizations
                              </span>
                            </>
                          )}
                        </div>
                      </div>
                    </div>

                    {model.files.length > 0 && (
                      <div className="mt-3 ml-9">
                        <div className="flex flex-wrap gap-2">
                          {Object.entries(quantGroups).map(([quant, files]) => {
                            const groupSize = files.reduce(
                              (sum, f) => sum + f.size,
                              0,
                            );
                            const isDeleting =
                              deletingQuant?.modelId === model.model_id &&
                              deletingQuant?.quant === quant;

                            return (
                              <div
                                key={quant}
                                className="group flex items-center gap-1.5 bg-white/5 border border-white/5 rounded px-2 py-1 hover:border-purple-accent/30 transition-all"
                              >
                                <span className="text-purple-accent/60 text-xs font-medium">
                                  {quant === "unquantized" ? "base" : quant}
                                </span>
                                <span className="text-white/20 text-xs">
                                  {formatFileSize(groupSize)}
                                </span>
                                {files.map((file) => (
                                  <span
                                    key={file.filename}
                                    className="text-white/40 text-xs font-mono"
                                  >
                                    {file.filename}
                                  </span>
                                ))}
                                {hasMultipleQuants && (
                                  <button
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      handleDeleteQuant(model.model_id, quant);
                                    }}
                                    disabled={!!deletingQuant}
                                    className="text-white/30 hover:text-error transition-colors cursor-pointer disabled:opacity-30"
                                    title={`Delete ${quant} quantization`}
                                  >
                                    {isDeleting ? (
                                      <RefreshCw
                                        size={14}
                                        className="animate-spin"
                                      />
                                    ) : (
                                      <Trash size={14} />
                                    )}
                                  </button>
                                )}
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    )}
                  </div>

                  <div className="ml-4 shrink-0">
                    {showDeleteConfirm === model.model_id ? (
                      <div className="flex items-center gap-2">
                        <span className="text-xs text-white/40">
                          Delete all?
                        </span>
                        <button
                          onClick={() => handleDelete(model.model_id)}
                          disabled={deleting === model.model_id}
                          className="px-3 py-1.5 bg-error hover:bg-error/80 text-white rounded-lg text-xs transition-all cursor-pointer disabled:opacity-50"
                        >
                          {deleting === model.model_id ? "Deleting..." : "Yes"}
                        </button>
                        <button
                          onClick={() => setShowDeleteConfirm(null)}
                          className="px-3 py-1.5 bg-white/10 hover:bg-white/20 text-white rounded-lg text-xs transition-all cursor-pointer"
                        >
                          No
                        </button>
                      </div>
                    ) : (
                      <button
                        onClick={() => setShowDeleteConfirm(model.model_id)}
                        className="text-white/30 hover:text-error transition-all p-2 rounded-lg hover:bg-error-bg cursor-pointer"
                        title="Delete entire model"
                      >
                        <Trash size={16} />
                      </button>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
