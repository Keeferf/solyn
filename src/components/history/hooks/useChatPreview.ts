import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ChatSession, ChatSessionWithMessages } from "../types";

export const useChatPreviews = (chats: ChatSession[]) => {
  const [previews, setPreviews] = useState<Map<number, string>>(new Map());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadPreviews = async () => {
      setLoading(true);
      setError(null);
      try {
        const newPreviews = new Map<number, string>();

        await Promise.all(
          chats.map(async (session) => {
            try {
              const chatData = await invoke<ChatSessionWithMessages | null>(
                "get_chat_session",
                { sessionId: session.id },
              );

              if (chatData?.messages.length) {
                const firstUserMessage = chatData.messages.find(
                  (msg) => msg.role === "user",
                );
                if (firstUserMessage) {
                  const firstLine = firstUserMessage.content.split("\n")[0];
                  newPreviews.set(session.id, firstLine);
                }
              }
            } catch {}
          }),
        );

        setPreviews(newPreviews);
      } catch (error) {
        setError("Failed to load chat previews");
      } finally {
        setLoading(false);
      }
    };

    if (chats.length > 0) {
      loadPreviews();
    } else {
      setPreviews(new Map());
      setLoading(false);
    }
  }, [chats]);

  const updatePreview = useCallback((chatId: number, newPreview: string) => {
    setPreviews((prev) => {
      const newMap = new Map(prev);
      newMap.set(chatId, newPreview);
      return newMap;
    });
  }, []);

  const removePreview = useCallback((chatId: number) => {
    setPreviews((prev) => {
      const newMap = new Map(prev);
      newMap.delete(chatId);
      return newMap;
    });
  }, []);

  return {
    previews,
    loading,
    error,
    updatePreview,
    removePreview,
  };
};
