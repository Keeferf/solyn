import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface GGUFFile {
  filename: string;
  size: number;
  quantization: string;
  url: string;
  parameter_count?: string | null;
}

export interface HFModelSummary {
  id: string;
  model_id: string;
  author: string;
  name: string;
  downloads?: number;
  likes?: number;
  created_at?: string;
  last_modified?: string;
}

export interface HFModelDetails extends HFModelSummary {
  description?: string;
  gguf_files: GGUFFile[];
}

export type HFModel = HFModelSummary | HFModelDetails;

export type ModelFilter = "most_downloads" | "most_liked" | "recent";

export function hasDetails(model: HFModel): model is HFModelDetails {
  return (
    "gguf_files" in model && Array.isArray((model as HFModelDetails).gguf_files)
  );
}

export const useHuggingFaceModels = (
  initialFilter: ModelFilter = "most_downloads",
) => {
  const [models, setModels] = useState<HFModelSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [totalModels, setTotalModels] = useState(0);
  const [currentFilter, setCurrentFilter] =
    useState<ModelFilter>(initialFilter);
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [isSearching, setIsSearching] = useState(false);

  const maxModels = 100;
  const initialLoadDone = useRef(false);
  const loadedIdsRef = useRef<Set<string>>(new Set());
  const isChangingFilterRef = useRef(false);
  const searchTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const loadInitialModels = useCallback(
    async (
      filter: ModelFilter,
      query: string = "",
      keepExistingModels: boolean = false,
    ) => {
      if (
        initialLoadDone.current &&
        filter === currentFilter &&
        query === searchQuery &&
        !isChangingFilterRef.current
      ) {
        return;
      }

      if (!keepExistingModels) {
        setLoading(true);
      }
      setError(null);

      try {
        let total = 0;

        try {
          if (query.trim()) {
            total = await invoke<number>("get_huggingface_search_count", {
              query: query.trim(),
              filter: filter,
            });
          } else {
            total = await invoke<number>("get_huggingface_model_count", {
              filter: filter,
            });
          }
        } catch (countErr) {
          total = maxModels;
        }

        const capped = Math.min(total, maxModels);
        setTotalModels(capped);

        let response: HFModelSummary[] = [];

        if (query.trim()) {
          const searchResult = await invoke<{
            models: HFModelSummary[];
            total: number;
            has_more: boolean;
          }>("search_huggingface_models", {
            query: query.trim(),
            page: 1,
            limit: maxModels,
            filter: filter,
          });

          response = searchResult.models || [];
        } else {
          response = await invoke<HFModelSummary[]>(
            "fetch_huggingface_models_page",
            {
              page: 1,
              limit: maxModels,
              filter: filter,
            },
          );
        }

        const ids = new Set<string>();
        response.forEach((m) => ids.add(m.model_id));
        loadedIdsRef.current = ids;

        setModels(response);
        setCurrentFilter(filter);
        setSearchQuery(query);

        initialLoadDone.current = true;
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    },
    [maxModels, currentFilter, searchQuery],
  );

  const changeFilter = useCallback(
    async (newFilter: ModelFilter) => {
      if (newFilter === currentFilter) return;

      loadedIdsRef.current = new Set();
      isChangingFilterRef.current = true;
      initialLoadDone.current = false;

      try {
        await loadInitialModels(newFilter, searchQuery, true);
      } catch (err) {
        setError(String(err));
      } finally {
        isChangingFilterRef.current = false;
        setLoading(false);
      }
    },
    [currentFilter, loadInitialModels, searchQuery],
  );

  const searchModels = useCallback(
    async (query: string) => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
        searchTimeoutRef.current = null;
      }

      if (query === searchQuery) return;

      searchTimeoutRef.current = setTimeout(async () => {
        setIsSearching(true);

        try {
          if (query.trim() !== "" || query !== searchQuery) {
            setModels([]);
            setTotalModels(0);
            loadedIdsRef.current = new Set();
            initialLoadDone.current = false;
            setSearchQuery(query);

            await loadInitialModels(currentFilter, query, false);
          }
        } catch (err) {
          setError(String(err));
        } finally {
          setLoading(false);
          setIsSearching(false);
        }
      }, 300);
    },
    [currentFilter, loadInitialModels, searchQuery],
  );

  const refreshModels = useCallback(async () => {
    initialLoadDone.current = false;
    loadedIdsRef.current = new Set();
    setModels([]);
    setTotalModels(0);

    try {
      await invoke("clear_models_cache", { filter: currentFilter });
    } catch {}

    await loadInitialModels(currentFilter, searchQuery, false);
  }, [loadInitialModels, currentFilter, searchQuery]);

  const clearSearch = useCallback(async () => {
    if (!searchQuery) return;

    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current);
      searchTimeoutRef.current = null;
    }

    setIsSearching(true);
    setSearchQuery("");

    setModels([]);
    setTotalModels(0);
    loadedIdsRef.current = new Set();
    initialLoadDone.current = false;

    await loadInitialModels(currentFilter, "", false);
    setIsSearching(false);
  }, [searchQuery, currentFilter, loadInitialModels]);

  useEffect(() => {
    return () => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }
    };
  }, []);

  useEffect(() => {
    loadInitialModels(initialFilter, "", false);
  }, []);

  return {
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
  };
};
