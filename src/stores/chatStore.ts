// src/stores/chatStore.ts
import { create } from "zustand";
import { persist } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";

export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

export interface ChatSession {
  id: number;
  title: string;
  model_name: string;
  created_at: string;
  updated_at: string;
}

export interface StoredChatMessage {
  id: number;
  session_id: number;
  role: string;
  content: string;
  created_at: string;
}

export interface ChatSessionWithMessages {
  session: ChatSession;
  messages: StoredChatMessage[];
}

interface ChatState {
  // Session state
  sessions: ChatSession[];
  currentSessionId: number | null;
  currentMessages: ChatMessage[];
  currentModelName: string | null;

  // Loading states
  isLoadingSessions: boolean;
  isLoadingMessages: boolean;
  isStreaming: boolean;
  error: string | null;

  // Actions
  loadSessions: () => Promise<void>;
  loadSession: (sessionId: number) => Promise<void>;
  createSession: (modelName: string, title?: string) => Promise<number>;
  deleteSession: (sessionId: number) => Promise<void>;
  updateSessionTitle: (sessionId: number, title: string) => Promise<void>;
  addMessage: (sessionId: number, message: ChatMessage) => Promise<void>;
  clearCurrentSession: () => void;
  startNewChat: () => void;
  setStreaming: (isStreaming: boolean) => void;
  setError: (error: string | null) => void;
  setCurrentMessages: (messages: ChatMessage[]) => void;
  setCurrentModelName: (modelName: string | null) => void;
  setLoadingSessions: (loading: boolean) => void;
  setLoadingMessages: (loading: boolean) => void;
}

export const useChatStore = create<ChatState>()(
  persist(
    (set, get) => ({
      // Initial state
      sessions: [],
      currentSessionId: null,
      currentMessages: [],
      currentModelName: null,
      isLoadingSessions: false,
      isLoadingMessages: false,
      isStreaming: false,
      error: null,

      // Load all chat sessions
      loadSessions: async () => {
        set({ isLoadingSessions: true, error: null });
        try {
          const sessions = await invoke<ChatSession[]>("get_chat_sessions");
          set({ sessions, isLoadingSessions: false });
        } catch (error) {
          set({
            error: "Failed to load chat sessions",
            isLoadingSessions: false,
          });
        }
      },

      // Load a specific session with its messages
      loadSession: async (sessionId: number) => {
        set({ isLoadingMessages: true, error: null });
        try {
          const result = await invoke<ChatSessionWithMessages | null>(
            "get_chat_session",
            { sessionId },
          );

          if (result) {
            const messages: ChatMessage[] = result.messages.map((m) => ({
              role: m.role as "user" | "assistant" | "system",
              content: m.content,
            }));

            // Update sessions list to ensure it's in sync
            const { sessions } = get();
            const sessionExists = sessions.some((s) => s.id === sessionId);
            if (!sessionExists) {
              set((state) => ({
                sessions: [result.session, ...state.sessions],
              }));
            }

            set({
              currentSessionId: sessionId,
              currentMessages: messages,
              currentModelName: result.session.model_name,
              isLoadingMessages: false,
              error: null,
            });
          } else {
            set({
              error: "Session not found",
              isLoadingMessages: false,
            });
          }
        } catch (error) {
          set({
            error: "Failed to load chat session",
            isLoadingMessages: false,
          });
        }
      },

      // Create a new chat session
      createSession: async (modelName: string, title?: string) => {
        try {
          const sessionId = await invoke<number>("create_chat_session", {
            request: {
              model_name: modelName,
              title: title || `Chat with ${modelName}`,
            },
          });

          // Reload sessions to get the new one
          await get().loadSessions();

          // Set as current session with empty messages
          set({
            currentSessionId: sessionId,
            currentMessages: [],
            currentModelName: modelName,
            error: null,
          });

          return sessionId;
        } catch (error) {
          set({ error: "Failed to create chat session" });
          throw error;
        }
      },

      // Delete a chat session
      deleteSession: async (sessionId: number) => {
        try {
          await invoke("delete_chat_session", { sessionId });

          // Remove from state
          const { currentSessionId } = get();
          set((state) => ({
            sessions: state.sessions.filter((s) => s.id !== sessionId),
            currentSessionId:
              currentSessionId === sessionId ? null : currentSessionId,
            currentMessages:
              currentSessionId === sessionId ? [] : state.currentMessages,
            currentModelName:
              currentSessionId === sessionId ? null : state.currentModelName,
          }));
        } catch (error) {
          set({ error: "Failed to delete chat session" });
          throw error;
        }
      },

      // Update session title
      updateSessionTitle: async (sessionId: number, title: string) => {
        try {
          await invoke("update_chat_session_title", { sessionId, title });

          // Update in state
          set((state) => ({
            sessions: state.sessions.map((s) =>
              s.id === sessionId ? { ...s, title } : s,
            ),
          }));
        } catch (error) {
          set({ error: "Failed to update session title" });
          throw error;
        }
      },

      // Add a message to the current session
      addMessage: async (sessionId: number, message: ChatMessage) => {
        try {
          // Add to local state first for immediate UI update
          set((state) => ({
            currentMessages: [...state.currentMessages, message],
          }));

          // If it's a user message, save to database immediately
          if (message.role === "user") {
            await invoke("add_message_to_session", {
              sessionId,
              message: {
                role: message.role,
                content: message.content,
              },
            });

            // Update sessions list to refresh timestamps
            await get().loadSessions();
          }
          // For assistant messages, they'll be saved by the backend via send_chat_stream

          set({ error: null });
        } catch (error) {
          // Rollback the message on error
          set((state) => ({
            currentMessages: state.currentMessages.filter((m) => m !== message),
            error: "Failed to save message",
          }));
          throw error;
        }
      },

      // Clear the current session
      clearCurrentSession: () => {
        set({
          currentSessionId: null,
          currentMessages: [],
          currentModelName: null,
          error: null,
          isStreaming: false,
        });
      },

      // Start a new chat (clear current messages but keep sessions)
      startNewChat: () => {
        set({
          currentSessionId: null,
          currentMessages: [],
          currentModelName: null,
          error: null,
          isStreaming: false,
        });
      },

      // Set streaming state
      setStreaming: (isStreaming: boolean) => {
        set({ isStreaming });
      },

      // Set error state
      setError: (error: string | null) => {
        set({ error });
      },

      // Set current messages (used for streaming updates)
      setCurrentMessages: (messages: ChatMessage[]) => {
        set({ currentMessages: messages });
      },

      // Set current model name
      setCurrentModelName: (modelName: string | null) => {
        set({ currentModelName: modelName });
      },

      // Set loading state for sessions
      setLoadingSessions: (loading: boolean) => {
        set({ isLoadingSessions: loading });
      },

      // Set loading state for messages
      setLoadingMessages: (loading: boolean) => {
        set({ isLoadingMessages: loading });
      },
    }),
    {
      name: "chat-storage",
      // Only persist sessions list and current session ID, not messages (they're heavy)
      partialize: (state) => ({
        sessions: state.sessions,
        currentSessionId: state.currentSessionId,
        currentModelName: state.currentModelName,
      }),
    },
  ),
);
