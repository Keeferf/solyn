import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useChatStore } from "@/stores/chatStore";

export type ModelType = string;

export interface ChatModel {
  value: string;
  label: string;
  model_id: string;
  author: string;
  name?: string;
  quantization?: string;
  parameter_count?: string;
  filename: string;
  path: string;
  has_modelfile: boolean;
  size?: number;
  ollama_model_name: string;
}

export const useModelSelection = () => {
  const [selectedModel, setSelectedModel] = useState<ModelType>("");
  const [isModelDropdownOpen, setIsModelDropdownOpen] = useState(false);
  const [models, setModels] = useState<ChatModel[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // Get the current model name from Zustand
  const { currentModelName } = useChatStore();

  const loadModels = useCallback(async () => {
    try {
      setIsLoading(true);
      const installedModels = await invoke<ChatModel[]>("get_chat_models");

      if (installedModels.length === 0) {
        setModels([
          {
            value: "no-models",
            label: "No models installed",
            model_id: "",
            author: "",
            filename: "",
            path: "",
            has_modelfile: false,
            ollama_model_name: "",
          },
        ]);
        setSelectedModel("no-models");
      } else {
        setModels(installedModels);

        // If we have a current model name from Zustand, try to select it
        if (currentModelName) {
          // Try to find the model by ollama_model_name, value, or label
          const matchedModel = installedModels.find(
            (m) =>
              m.ollama_model_name === currentModelName ||
              m.value === currentModelName ||
              m.label === currentModelName,
          );

          if (matchedModel) {
            setSelectedModel(matchedModel.value);
          } else {
            // Fallback to first model
            setSelectedModel(installedModels[0].value);
          }
        } else {
          // Check if current selection exists
          const currentModelExists = installedModels.some(
            (m) => m.value === selectedModel,
          );

          if (
            !currentModelExists ||
            selectedModel === "" ||
            selectedModel === "no-models"
          ) {
            setSelectedModel(installedModels[0].value);
          }
        }
      }
    } catch (error) {
      setModels([
        {
          value: "error",
          label: "Error loading models",
          model_id: "",
          author: "",
          filename: "",
          path: "",
          has_modelfile: false,
          ollama_model_name: "",
        },
      ]);
      setSelectedModel("error");
    } finally {
      setIsLoading(false);
    }
  }, [currentModelName, selectedModel]);

  // When currentModelName changes, update the selected model
  useEffect(() => {
    if (
      currentModelName &&
      models.length > 0 &&
      models[0].value !== "no-models" &&
      models[0].value !== "error"
    ) {
      const matchedModel = models.find(
        (m) =>
          m.ollama_model_name === currentModelName ||
          m.value === currentModelName ||
          m.label === currentModelName,
      );

      if (matchedModel && matchedModel.value !== selectedModel) {
        setSelectedModel(matchedModel.value);
      }
    }
  }, [currentModelName, models, selectedModel]);

  useEffect(() => {
    loadModels();
  }, [loadModels]);

  useEffect(() => {
    let unlistenFn: (() => void) | undefined;

    const setupListener = async () => {
      try {
        const unlisten = await listen("model-download-complete", () => {
          loadModels();
        });
        unlistenFn = unlisten;
      } catch {}
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [loadModels]);

  const selectModel = (model: ModelType) => {
    setSelectedModel(model);
    setIsModelDropdownOpen(false);
  };

  const toggleDropdown = () => {
    setIsModelDropdownOpen(!isModelDropdownOpen);
  };

  const closeDropdown = () => {
    setIsModelDropdownOpen(false);
  };

  const getSelectedModelData = (): ChatModel | undefined => {
    return models.find((m) => m.value === selectedModel);
  };

  return {
    selectedModel,
    models,
    isModelDropdownOpen,
    isLoading,
    selectModel,
    toggleDropdown,
    closeDropdown,
    getSelectedModelData,
    loadModels,
  };
};
