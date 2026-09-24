import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface OllamaStatus {
  installed: boolean;
  running: boolean;
  version: string | null;
}

interface OllamaContextType {
  status: OllamaStatus | null;
  loading: boolean;
  refreshing: boolean;
  refreshOllamaStatus: () => Promise<void>;
  startOllama: () => Promise<void>;
  isReady: boolean;
}

const OllamaContext = createContext<OllamaContextType | undefined>(undefined);

export const OllamaProvider = ({ children }: { children: ReactNode }) => {
  const [status, setStatus] = useState<OllamaStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  const refreshOllamaStatus = async () => {
    setRefreshing(true);
    try {
      const result = await invoke<OllamaStatus>("refresh_ollama_status");
      setStatus(result);
    } catch (error) {
      setStatus({
        installed: false,
        running: false,
        version: null,
      });
    } finally {
      setRefreshing(false);
      setLoading(false);
    }
  };

  const startOllama = async () => {
    if (!status?.installed) {
      throw new Error("Ollama is not installed");
    }

    if (status.running) {
      return;
    }

    setLoading(true);
    try {
      await invoke<string>("start_ollama_service");
      await refreshOllamaStatus();
    } catch (error) {
      throw error;
    } finally {
      setLoading(false);
    }
  };

  // Listen for status updates from the backend
  useEffect(() => {
    // Initial status check
    refreshOllamaStatus();

    // Listen for status updates via events
    const unlisten = listen<OllamaStatus>("ollama-status-update", (event) => {
      setStatus(event.payload);
      setLoading(false);
      setRefreshing(false);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const isReady = (status?.installed && status?.running) ?? false;

  return (
    <OllamaContext.Provider
      value={{
        status,
        loading,
        refreshing,
        refreshOllamaStatus,
        startOllama,
        isReady,
      }}
    >
      {children}
    </OllamaContext.Provider>
  );
};

export const useOllama = () => {
  const context = useContext(OllamaContext);
  if (!context) {
    throw new Error("useOllama must be used within an OllamaProvider");
  }
  return context;
};
