import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ChatSession, ChatSessionWithMessages } from "../types";

export const useChatHistory = () => {
  const [chats, setChats] = useState<ChatSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadChats = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const sessions = await invoke<ChatSession[]>("get_chat_sessions");
      setChats(sessions);
      return sessions;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to load chats";
      setError(message);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  const getChatSession = useCallback(async (sessionId: number) => {
    try {
      return await invoke<ChatSessionWithMessages | null>("get_chat_session", {
        sessionId,
      });
    } catch (error) {
      throw error;
    }
  }, []);

  const renameChat = useCallback(async (sessionId: number, title: string) => {
    try {
      await invoke("update_chat_session_title", { sessionId, title });
      setChats((prev) =>
        prev.map((chat) => (chat.id === sessionId ? { ...chat, title } : chat)),
      );
      return true;
    } catch (error) {
      throw error;
    }
  }, []);

  const deleteChat = useCallback(async (sessionId: number) => {
    try {
      await invoke("delete_chat_session", { sessionId });
      setChats((prev) => prev.filter((chat) => chat.id !== sessionId));
      return true;
    } catch (error) {
      throw error;
    }
  }, []);

  const pinChat = useCallback(async (_sessionId: number) => {}, []);

  useEffect(() => {
    loadChats();
  }, [loadChats]);

  return {
    chats,
    loading,
    error,
    loadChats,
    getChatSession,
    renameChat,
    deleteChat,
    pinChat,
  };
};
