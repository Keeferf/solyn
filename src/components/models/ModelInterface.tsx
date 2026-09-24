import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BrowseModels } from "./BrowseModels";
import { InstalledModels } from "./InstalledModels";
import { ModelToolbar } from "./ModelToolbar";
import { DownloadStatusDisplay } from "./DownloadStatusDisplay";
import { ModelDetailModal } from "./ModelDetailModal";
import {
  useHuggingFaceModels,
  HFModelSummary,
} from "./hooks/useHuggingFaceModels";

interface DownloadProgress {
  model_id: string;
  filename: string;
  status: string;
  progress: number;
  message: string;
}

type DownloadKey = string;

export const ModelInterface = () => {
  const [activeTab, setActiveTab] = useState<"browse" | "installed">("browse");

  const {
    models,
    loading,
    error,
    totalModels,
    maxModels,
    currentFilter,
    searchQuery,
    isSearching,
    changeFilter,
    refreshModels,
    searchModels,
    clearSearch,
  } = useHuggingFaceModels();

  const [selectedModelId, setSelectedModelId] = useState<string | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [downloadingModels, setDownloadingModels] = useState<Set<DownloadKey>>(
    new Set(),
  );
  const [downloadProgress, setDownloadProgress] = useState<
    Map<DownloadKey, DownloadProgress>
  >(new Map());
  const [cancellingDownloads, setCancellingDownloads] = useState<
    Set<DownloadKey>
  >(new Set());

  const getDownloadKey = (modelId: string, filename: string): DownloadKey => {
    return `${modelId}::${filename}`;
  };

  const isDownloading = (modelId: string, filename: string): boolean => {
    return downloadingModels.has(getDownloadKey(modelId, filename));
  };

  const handleCancelDownload = (modelId: string, filename: string) => {
    const key = getDownloadKey(modelId, filename);
    setCancellingDownloads((prev) => new Set(prev).add(key));
  };

  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenComplete: (() => void) | undefined;

    const setupListeners = async () => {
      try {
        unlistenProgress = await listen<DownloadProgress>(
          "model-download-progress",
          (event) => {
            const progress = event.payload;
            const key = getDownloadKey(progress.model_id, progress.filename);

            setCancellingDownloads((prev) => {
              const newSet = new Set(prev);
              newSet.delete(key);
              return newSet;
            });

            if (
              !downloadingModels.has(key) &&
              (progress.status === "starting" ||
                progress.status === "downloading")
            ) {
              setDownloadingModels((prev) => new Set(prev).add(key));
            }

            setDownloadProgress((prev) => new Map(prev).set(key, progress));

            if (
              progress.status === "complete" ||
              progress.status === "error" ||
              progress.status === "cancelled"
            ) {
              const delay = progress.status === "cancelled" ? 5000 : 3000;
              setTimeout(() => {
                setDownloadingModels((prev) => {
                  const newSet = new Set(prev);
                  newSet.delete(key);
                  return newSet;
                });
                setDownloadProgress((prev) => {
                  const newMap = new Map(prev);
                  newMap.delete(key);
                  return newMap;
                });
                setCancellingDownloads((prev) => {
                  const newSet = new Set(prev);
                  newSet.delete(key);
                  return newSet;
                });
              }, delay);
            }
          },
        );

        unlistenComplete = await listen<{ model_id: string; filename: string }>(
          "model-download-complete",
          (event) => {
            const { model_id, filename } = event.payload;
            const key = getDownloadKey(model_id, filename);

            setDownloadProgress((prev) => {
              const existing = prev.get(key);
              if (existing) {
                const newMap = new Map(prev);
                newMap.set(key, {
                  ...existing,
                  status: "complete",
                  progress: 100,
                  message: "Download complete!",
                });
                return newMap;
              }
              return prev;
            });

            setTimeout(() => {
              setDownloadingModels((prev) => {
                const newSet = new Set(prev);
                newSet.delete(key);
                return newSet;
              });
              setDownloadProgress((prev) => {
                const newMap = new Map(prev);
                newMap.delete(key);
                return newMap;
              });
            }, 3000);
          },
        );
      } catch {}
    };

    setupListeners();

    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenComplete) unlistenComplete();
    };
  }, [downloadingModels]);

  const handleModelClick = (model: HFModelSummary) => {
    setSelectedModelId(model.model_id);
    setIsModalOpen(true);
  };

  const handleDownload = async (modelId: string, filename: string) => {
    const key = getDownloadKey(modelId, filename);
    if (downloadingModels.has(key) || cancellingDownloads.has(key)) return;

    setDownloadingModels((prev) => new Set(prev).add(key));

    try {
      await invoke("download_huggingface_model", {
        modelId,
        filename,
      });
    } catch (error) {
      if (error !== "Download cancelled") {
        setDownloadingModels((prev) => {
          const newSet = new Set(prev);
          newSet.delete(key);
          return newSet;
        });
        setDownloadProgress((prev) => {
          const newMap = new Map(prev);
          newMap.delete(key);
          return newMap;
        });
      }
    }
  };

  const handleCloseModal = () => {
    setIsModalOpen(false);
    setSelectedModelId(null);
  };

  const handleFilterChange = (filter: string) => {
    changeFilter(filter as any);
  };

  const handleSearchChange = (query: string) => {
    searchModels(query);
  };

  const handleClearSearch = () => {
    clearSearch();
  };

  return (
    <div className="w-full h-full px-4">
      <div className="flex items-center justify-between pt-4 pb-2">
        <div className="flex items-center gap-8">
          <button
            onClick={() => setActiveTab("browse")}
            className={`font-anton text-3xl sm:text-4xl tracking-wide transition-all cursor-pointer ${
              activeTab === "browse"
                ? "text-white"
                : "text-white/30 hover:text-white/50"
            }`}
          >
            Browse
          </button>
          <button
            onClick={() => setActiveTab("installed")}
            className={`font-anton text-3xl sm:text-4xl tracking-wide transition-all cursor-pointer ${
              activeTab === "installed"
                ? "text-white"
                : "text-white/30 hover:text-white/50"
            }`}
          >
            Installed
          </button>
        </div>
        {activeTab === "browse" && (
          <button
            onClick={refreshModels}
            disabled={loading || isSearching}
            className="px-4 py-2 bg-black hover:bg-white/10 rounded-lg text-white transition-all disabled:opacity-50 cursor-pointer"
          >
            Refresh
          </button>
        )}
      </div>

      <div className="pb-6 space-y-6">
        {Array.from(downloadProgress.entries()).map(([key, progress]) => (
          <DownloadStatusDisplay
            key={key}
            modelId={progress.model_id}
            filename={progress.filename}
            progress={progress.progress}
            message={progress.message}
            status={progress.status}
            onCancel={() =>
              handleCancelDownload(progress.model_id, progress.filename)
            }
          />
        ))}

        {activeTab === "browse" ? (
          <>
            <ModelToolbar
              searchQuery={searchQuery}
              onSearchChange={handleSearchChange}
              onClearSearch={handleClearSearch}
              currentFilter={currentFilter}
              onFilterChange={handleFilterChange}
              loading={loading || isSearching}
            />

            <BrowseModels
              models={models}
              loading={loading}
              isSearching={isSearching}
              totalModels={totalModels}
              maxModels={maxModels}
              downloadingModels={downloadingModels}
              onModelClick={handleModelClick}
              onRefresh={refreshModels}
              error={error}
              searchQuery={searchQuery}
              onClearSearch={handleClearSearch}
            />
          </>
        ) : (
          <InstalledModels
            onModelClick={() => {}}
          />
        )}
      </div>

      <ModelDetailModal
        modelId={selectedModelId}
        isOpen={isModalOpen}
        onClose={handleCloseModal}
        onDownload={handleDownload}
        downloadingModels={downloadingModels}
        isDownloading={isDownloading}
      />
    </div>
  );
};
