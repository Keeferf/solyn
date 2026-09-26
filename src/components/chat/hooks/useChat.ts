import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useOllama } from "@/contexts/OllamaContext";
import { useChatStore, SessionSettings } from "@/stores/chatStore";

export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
  thinking?: string;
}

export interface ChatModelData {
  model_id: string;
  filename: string;
  ollama_model_name: string;
}

export const useChat = (
  modelData: ChatModelData | undefined,
  settings: SessionSettings,
) => {
  const [isLoading, setIsLoading] = useState(false);
  const [modelLoaded, setModelLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const {
    currentSessionId,
    currentMessages,
    setCurrentMessages,
    setStreaming,
    addMessage,
    loadSessions,
    loadSession,
    createSession: createSessionStore,
    setError: setStoreError,
    startNewChat,
    currentModelName,
    setCurrentModelName,
  } = useChatStore();

  const unlistenRefs = useRef<UnlistenFn[]>([]);
  const inFlightRef = useRef(false);
  const { isReady } = useOllama();
  const isOllamaReady = isReady;
  const currentMessagesRef = useRef(currentMessages);

  // Keep ref in sync with currentMessages
  useEffect(() => {
    currentMessagesRef.current = currentMessages;
  }, [currentMessages]);

  const loadSessionWithMessages = useCallback(
    async (sessionId: number) => {
      await loadSession(sessionId);
    },
    [loadSession],
  );

  const createSession = useCallback(
    async (modelName: string, title?: string) => {
      return await createSessionStore(modelName, title);
    },
    [createSessionStore],
  );

  const sendMessage = async (content: string) => {
    if (!content.trim() || isLoading || !modelData) {
      return;
    }

    if (!isOllamaReady) {
      const errorMsg = "Ollama is not running. Please wait for it to start.";
      setError(errorMsg);
      setStoreError(errorMsg);
      return;
    }

    if (!modelData.ollama_model_name) {
      const errorMsg =
        "Selected model has no Ollama name. Please reinstall the model.";
      setError(errorMsg);
      setStoreError(errorMsg);
      return;
    }


    setError(null);
    setStoreError(null);

    // Guard against a second submit racing between click and re-render.
    if (inFlightRef.current) return;
    inFlightRef.current = true;

    setIsLoading(true);
    setStreaming(true);

    let sessionId = currentSessionId;
    if (!sessionId) {
      try {
        // Use the first message as the title (truncated to 50 chars)
        const title =
          content.trim().slice(0, 50) +
          (content.trim().length > 50 ? "..." : "");
        sessionId = await createSessionStore(
          modelData.ollama_model_name,
          title,
        );
      } catch (err) {
        const errorMsg = "Failed to create chat session";
        setError(errorMsg);
        setStoreError(errorMsg);
        setIsLoading(false);
        setStreaming(false);
        inFlightRef.current = false;
        return;
      }
    }

    // Optimistic UI: show the user turn and an empty assistant placeholder.
    // The backend owns the transcript (send_chat_stream persists both turns),
    // so we do not write the user message from here.
    const userMessage: ChatMessage = { role: "user", content: content.trim() };
    const assistantMessage: ChatMessage = { role: "assistant", content: "" };
    const nextMessages = [
      ...currentMessagesRef.current,
      userMessage,
      assistantMessage,
    ];
    currentMessagesRef.current = nextMessages;
    setCurrentMessages(nextMessages);

    try {
      await invoke("send_chat_stream", {
        request: {
          model: modelData.ollama_model_name,
          message: content.trim(),
          session_id: sessionId,
          settings,
        },
      });

      await loadSessions();
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      setStoreError(errorMsg);
      setIsLoading(false);
      setStreaming(false);

      // Remove the empty assistant message on error
      const messages = currentMessagesRef.current;
      if (
        messages.length > 0 &&
        messages[messages.length - 1].role === "assistant" &&
        messages[messages.length - 1].content === ""
      ) {
        const updatedMessages = messages.slice(0, -1);
        currentMessagesRef.current = updatedMessages;
        setCurrentMessages(updatedMessages);
      }
    } finally {
      inFlightRef.current = false;
    }
  };

  const clearMessages = useCallback(() => {
    setCurrentMessages([]);
    setError(null);
    setStoreError(null);
  }, [setCurrentMessages, setStoreError]);

  // Set up event listeners for streaming
  useEffect(() => {
    const setupListeners = async () => {
      try {
        // Clean up old listeners
        unlistenRefs.current.forEach((unlisten) => {
          try {
            unlisten();
          } catch {}
        });
        unlistenRefs.current = [];


        // Listen for streaming chunks
        const unlistenChunk = await listen<{ chunk: string }>(
          "chat-stream-chunk",
          (event) => {
            const fullContent = event.payload.chunk;

            // Get current messages from the store
            const currentMessages = useChatStore.getState().currentMessages;
            const updatedMessages = [...currentMessages];
            const lastIndex = updatedMessages.length - 1;


            if (
              lastIndex >= 0 &&
              updatedMessages[lastIndex].role === "assistant"
            ) {
              updatedMessages[lastIndex] = {
                ...updatedMessages[lastIndex],
                content: fullContent,
              };
              // Update the store
              useChatStore.getState().setCurrentMessages(updatedMessages);
            }
          },
        );
        unlistenRefs.current.push(unlistenChunk);

        // Listen for streaming reasoning chunks
        const unlistenThinking = await listen<{ chunk: string }>(
          "chat-stream-thinking",
          (event) => {
            const thinking = event.payload.chunk;

            const currentMessages = useChatStore.getState().currentMessages;
            const updatedMessages = [...currentMessages];
            const lastIndex = updatedMessages.length - 1;

            if (
              lastIndex >= 0 &&
              updatedMessages[lastIndex].role === "assistant"
            ) {
              updatedMessages[lastIndex] = {
                ...updatedMessages[lastIndex],
                thinking,
              };
              useChatStore.getState().setCurrentMessages(updatedMessages);
            }
          },
        );
        unlistenRefs.current.push(unlistenThinking);

        // Listen for cold/warm model status (whether model had to be loaded)
        const unlistenModelStatus = await listen<{ loaded: boolean }>(
          "chat-stream-model-status",
          (event) => setModelLoaded(event.payload.loaded),
        );
        unlistenRefs.current.push(unlistenModelStatus);

        // Listen for stream completion
        const unlistenDone = await listen("chat-stream-done", () => {
          setIsLoading(false);
          setStreaming(false);
        });
        unlistenRefs.current.push(unlistenDone);

        // Listen for stream complete with final response
        const unlistenComplete = await listen<{
          response: string;
          thinking?: string;
        }>("chat-stream-complete", (event) => {
            const { response, thinking } = event.payload;

            // Ensure the final response is in the store
            const currentMessages = useChatStore.getState().currentMessages;
            const updatedMessages = [...currentMessages];
            const lastIndex = updatedMessages.length - 1;

            if (
              lastIndex >= 0 &&
              updatedMessages[lastIndex].role === "assistant"
            ) {
              // If the content or reasoning is different, update it
              const nextThinking =
                thinking ?? updatedMessages[lastIndex].thinking;
              if (
                updatedMessages[lastIndex].content !== response ||
                updatedMessages[lastIndex].thinking !== nextThinking
              ) {
                updatedMessages[lastIndex] = {
                  ...updatedMessages[lastIndex],
                  content: response,
                  thinking: nextThinking,
                };
                useChatStore.getState().setCurrentMessages(updatedMessages);
              }
            }

            setIsLoading(false);
            setStreaming(false);
          },
        );
        unlistenRefs.current.push(unlistenComplete);

        // Listen for stream errors
        const unlistenError = await listen<{ error: string }>(
          "chat-stream-error",
          (event) => {
            const { error: errorMsg } = event.payload;
            setError(errorMsg);
            setStoreError(errorMsg);
            setIsLoading(false);
            setStreaming(false);
          },
        );
        unlistenRefs.current.push(unlistenError);

      } catch {}
    };

    setupListeners();

    return () => {
      unlistenRefs.current.forEach((unlisten) => {
        try {
          unlisten();
        } catch {}
      });
      unlistenRefs.current = [];
    };
  }, [setStreaming, setStoreError]);

  return {
    messages: currentMessages,
    isLoading,
    isStreaming: useChatStore.getState().isStreaming,
    modelLoaded,
    error,
    isOllamaReady,
    currentSessionId,
    currentModelName,
    sendMessage,
    clearMessages,
    startNewChat,
    loadSessions,
    loadSession: loadSessionWithMessages,
    createSession,
    deleteSession: useChatStore.getState().deleteSession,
    updateSessionTitle: useChatStore.getState().updateSessionTitle,
    addMessage,
    setCurrentModelName,
  };
};
