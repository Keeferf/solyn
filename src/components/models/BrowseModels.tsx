import {
  Loader,
  Server,
  User,
  RefreshCw,
  Heart,
  CloudDownload,
  ChevronRight,
} from "lucide-react";
import { HFModelSummary } from "./hooks/useHuggingFaceModels";

interface BrowseModelsProps {
  models: HFModelSummary[];
  loading: boolean;
  totalModels: number;
  maxModels: number;
  downloadingModels: Set<string>;
  onModelClick: (model: HFModelSummary) => void;
  onRefresh?: () => void;
  error?: string | null;
  searchQuery?: string;
  isSearching?: boolean;
  onClearSearch?: () => void;
}

const formatDownloads = (downloads?: number): string => {
  if (!downloads) return "0";
  if (downloads >= 1_000_000) {
    return `${(downloads / 1_000_000).toFixed(1)}M`;
  }
  if (downloads >= 1_000) {
    return `${(downloads / 1_000).toFixed(1)}K`;
  }
  return downloads.toString();
};

const formatLikes = (likes?: number): string => {
  if (!likes) return "0";
  if (likes >= 1_000_000) {
    return `${(likes / 1_000_000).toFixed(1)}M`;
  }
  if (likes >= 1_000) {
    const roundedDown = Math.floor(likes / 100) * 100;
    const thousands = roundedDown / 1000;
    if (Number.isInteger(thousands)) {
      return `${thousands}k`;
    }
    return `${thousands.toFixed(1)}k`;
  }
  return likes.toString();
};

export const BrowseModels = ({
  models,
  loading,
  totalModels,
  maxModels,
  downloadingModels,
  onModelClick,
  onRefresh,
  error,
  searchQuery = "",
  isSearching = false,
  onClearSearch,
}: BrowseModelsProps) => {
  if (loading && models.length === 0) {
    return (
      <div className="w-full space-y-6">
        <div className="flex items-center justify-between text-sm text-white/40 px-2 py-2 bg-black/50 rounded-lg border border-white/5">
          <div className="flex items-center gap-3">
            <Server className="text-purple-accent" size={16} />
            <span>Loading GGUF models from Hugging Face...</span>
          </div>
        </div>
        <div className="flex items-center justify-center py-16">
          <Loader className="animate-spin text-purple-accent" size={40} />
        </div>
      </div>
    );
  }

  if (error && models.length === 0) {
    return (
      <div className="w-full space-y-6">
        <div className="flex items-center justify-between text-sm text-white/40 px-2 py-2 bg-black/50 rounded-lg border border-white/5">
          <div className="flex items-center gap-3">
            <Server className="text-purple-accent" size={16} />
            <span>Error loading GGUF models</span>
          </div>
        </div>
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <p className="text-white text-lg mb-2">Failed to load models</p>
          <p className="text-white/40 text-sm mb-4">{error}</p>
          {onRefresh && (
            <button
              onClick={onRefresh}
              className="px-4 py-2 bg-black hover:bg-white/10 rounded-lg text-white transition-all flex items-center gap-2 cursor-pointer"
            >
              <RefreshCw size={16} />
              Retry
            </button>
          )}
        </div>
      </div>
    );
  }

  if (!loading && models.length === 0) {
    return (
      <div className="w-full">
        <div className="text-center py-16">
          {searchQuery ? (
            <>
              <p className="text-white/40 text-lg">No models found</p>
              <p className="text-white/30 text-sm mt-2">
                No models match "{searchQuery}"
              </p>
              {onClearSearch && (
                <button
                  onClick={onClearSearch}
                  className="mt-4 px-4 py-2 bg-black hover:bg-white/10 rounded-lg text-white transition-all flex items-center gap-2 mx-auto cursor-pointer"
                >
                  <span>Clear search</span>
                </button>
              )}
            </>
          ) : (
            <>
              <p className="text-white/40 text-lg">No models available</p>
              <p className="text-white/30 text-sm mt-2">
                Try refreshing or check your connection
              </p>
              {onRefresh && (
                <button
                  onClick={onRefresh}
                  className="mt-4 px-4 py-2 bg-black hover:bg-white/10 rounded-lg text-white transition-all flex items-center gap-2 mx-auto cursor-pointer"
                >
                  <RefreshCw size={16} />
                  Refresh
                </button>
              )}
            </>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="w-full">
      <div
        className={`transition-all duration-500 ease-in-out ${
          isSearching ? "opacity-60 scale-[0.99]" : "opacity-100 scale-100"
        }`}
      >
        <div className="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-4">
          {models.map((model, index) => {
            const isDownloading = downloadingModels.has(model.model_id);

            return (
              <div
                key={`${model.id}-${index}`}
                onClick={() => onModelClick(model)}
                className={`group bg-black border border-white/10 rounded-xl p-5 transition-all duration-200 flex flex-col h-full hover:bg-white/5 hover:border-white/20 cursor-pointer hover:scale-[1.02] ${
                  isDownloading ? "opacity-50 pointer-events-none" : ""
                }`}
              >
                <div className="flex items-start justify-between mb-3">
                  <div className="flex-1 min-w-0">
                    <h4 className="text-white font-semibold truncate text-base">
                      {model.name || model.model_id}
                    </h4>
                    <div className="flex items-center gap-2 mt-1">
                      <User className="text-white/30" size={12} />
                      <span className="text-white/40 text-xs">
                        {model.author || "Unknown"}
                      </span>
                    </div>
                  </div>
                  {isDownloading && (
                    <div className="text-purple-accent">
                      <Loader className="animate-spin" size={16} />
                    </div>
                  )}
                </div>

                <div className="flex flex-wrap gap-2 mb-3">
                  {model.downloads !== undefined && model.downloads > 0 && (
                    <span className="text-xs bg-success-bg text-success px-2 py-1 rounded-full border border-success-border flex items-center gap-1">
                      <CloudDownload size={12} />
                      {formatDownloads(model.downloads)}
                    </span>
                  )}
                  {model.likes !== undefined && model.likes > 0 && (
                    <span className="text-xs bg-error-bg text-error px-2 py-1 rounded-full border border-error-border flex items-center gap-1">
                      <Heart size={12} />
                      {formatLikes(model.likes)}
                    </span>
                  )}
                </div>

                <div className="flex items-center justify-end mt-auto pt-3 border-t border-white/5">
                  <span className="text-xs text-white/30 flex items-center gap-1 group-hover:text-white/60 transition-colors">
                    View all quantizations
                    <ChevronRight
                      size={14}
                      className="text-white/20 group-hover:text-purple-accent/60 transition-all group-hover:translate-x-1"
                    />
                  </span>
                </div>
              </div>
            );
          })}
        </div>

        <div className="py-8 text-center text-white/30 text-sm border-t border-white/5">
          {models.length === totalModels && totalModels > 0 ? (
            <>
              Showing all {models.length} models
              {totalModels < maxModels && (
                <span className="block text-xs text-white/20 mt-2">
                  ({totalModels} available from Hugging Face)
                </span>
              )}
            </>
          ) : (
            <>
              Showing {models.length} of {totalModels} models
              {totalModels < maxModels && (
                <span className="block text-xs text-white/20 mt-2">
                  ({totalModels} available from Hugging Face)
                </span>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};
