import { ChevronDown } from "lucide-react";
import { ModelType, ChatModel } from "./hooks/useModelSelection";

interface ModelSelectorProps {
  selectedModel: ModelType;
  models: ChatModel[];
  isOpen: boolean;
  isLoading: boolean;
  onToggle: () => void;
  onSelect: (model: ModelType) => void;
  onClose: () => void;
}

export const ModelSelector = ({
  selectedModel,
  models,
  isOpen,
  isLoading,
  onToggle,
  onSelect,
  onClose,
}: ModelSelectorProps) => {
  const selectedLabel =
    models.find((m) => m.value === selectedModel)?.label || "Select model";
  const groupedModels = models.reduce(
    (acc, model) => {
      if (!acc[model.model_id]) {
        acc[model.model_id] = [];
      }
      acc[model.model_id].push(model);
      return acc;
    },
    {} as Record<string, ChatModel[]>,
  );

  return (
    <div className="relative">
      <button
        onClick={onToggle}
        disabled={isLoading}
        className="flex items-center gap-1.5 px-2 py-1 text-xs text-white/80 hover:text-white bg-white/5 hover:bg-white/10 rounded-lg transition-colors cursor-pointer h-8 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <span>{isLoading ? "Loading..." : selectedLabel}</span>
        <ChevronDown
          size={12}
          className={`transition-transform ${isOpen ? "rotate-180" : ""}`}
        />
      </button>

      {isOpen && !isLoading && (
        <>
          <div className="fixed inset-0 z-10" onClick={onClose} />
          <div className="absolute bottom-full mb-2 right-0 z-20 bg-black border border-white/10 rounded-lg shadow-lg py-1 min-w-60 max-h-87.5 overflow-y-auto">
            {Object.entries(groupedModels).map(([modelId, modelVariants]) => {
              const isPlaceholder =
                modelId === "no-models" || modelId === "error";

              if (isPlaceholder) {
                const model = modelVariants[0];
                return (
                  <button
                    key={model.value}
                    onClick={() => onSelect(model.value)}
                    disabled={true}
                    className="w-full text-left px-3 py-1.5 text-xs opacity-50 cursor-not-allowed text-white/60"
                  >
                    {model.label}
                  </button>
                );
              }

              if (modelVariants.length > 1) {
                return (
                  <div key={modelId}>
                    <div className="px-3 py-1 text-[10px] text-white/30 uppercase tracking-wider border-b border-white/5">
                      {modelVariants[0].name || modelVariants[0].author}
                    </div>
                    {modelVariants.map((model) => {
                      const isSelected = selectedModel === model.value;
                      return (
                        <button
                          key={model.value}
                          onClick={() => onSelect(model.value)}
                          className={`w-full text-left px-3 py-1.5 text-xs hover:bg-white/5 transition-colors cursor-pointer ${
                            isSelected
                              ? "text-white bg-white/5"
                              : "text-white/60"
                          }`}
                        >
                          <div className="flex items-center justify-between">
                            <span>{model.quantization || "Default"}</span>
                            {model.parameter_count && (
                              <span className="text-[10px] text-white/30">
                                {model.parameter_count}
                              </span>
                            )}
                          </div>
                        </button>
                      );
                    })}
                  </div>
                );
              } else {
                const model = modelVariants[0];
                const isSelected = selectedModel === model.value;
                return (
                  <button
                    key={model.value}
                    onClick={() => onSelect(model.value)}
                    className={`w-full text-left px-3 py-1.5 text-xs hover:bg-white/5 transition-colors cursor-pointer ${
                      isSelected ? "text-white bg-white/5" : "text-white/60"
                    }`}
                  >
                    <div className="flex flex-col">
                      <span>{model.label}</span>
                      {model.quantization && (
                        <span className="text-[10px] text-white/40">
                          {model.quantization}
                          {model.parameter_count &&
                            ` • ${model.parameter_count}`}
                        </span>
                      )}
                    </div>
                  </button>
                );
              }
            })}
          </div>
        </>
      )}
    </div>
  );
};
