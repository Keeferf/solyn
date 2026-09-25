// src/components/chat/ChatInterface.tsx
import { useState, useEffect } from "react";
import { FiX, FiPlus, FiEdit2, FiTrash2 } from "react-icons/fi";
import { ChatInput } from "./ChatInput";
import { ChatControls } from "./ChatControls";
import { ChatMessages } from "./ChatMessages";
import { useChatInput } from "./hooks/useChatInput";
import { useFileAttachment } from "./hooks/useFileAttachment";
import { useModelSelection } from "./hooks/useModelSelection";
import { useChat } from "./hooks/useChat";
import { useChatStore } from "@/stores/chatStore";

export type ModeType = "chat" | "agent";

export const ChatInterface = () => {
  const [isSearchEnabled, setIsSearchEnabled] = useState(false);
  const [isCodeEnabled, setIsCodeEnabled] = useState(false);
  const [mode, setMode] = useState<ModeType>("chat");
  const [showHistory, setShowHistory] = useState(false);

  // Zustand state
  const {
    sessions,
    currentSessionId,
    currentMessages,
    isLoadingSessions,
    error: storeError,
    isStreaming: storeIsStreaming,
    loadSessions,
    loadSession,
    deleteSession: deleteSessionStore,
    updateSessionTitle: updateSessionTitleStore,
    startNewChat: startNewChatStore,
    currentModelName,
  } = useChatStore();

  const { input, setInput, textareaRef, resetInput } = useChatInput();
  const {
    isAttachmentEnabled,
    attachments,
    fileInputRef,
    handleAttachmentClick,
    handleFileChange,
    clearAttachments,
  } = useFileAttachment();
  const {
    selectedModel,
    models,
    isModelDropdownOpen,
    isLoading: isModelsLoading,
    selectModel,
    toggleDropdown,
    closeDropdown,
    getSelectedModelData,
    loadModels,
  } = useModelSelection();

  const selectedModelData = getSelectedModelData();

  // Use the chat hook
  const {
    isLoading: isChatLoading,
    error: hookError,
    isOllamaReady,
    modelLoaded,
    sendMessage,
    startNewChat,
  } = useChat(
    selectedModelData && selectedModelData.ollama_model_name
      ? {
          model_id: selectedModelData.model_id,
          filename: selectedModelData.filename,
          ollama_model_name: selectedModelData.ollama_model_name,
        }
      : undefined,
  );

  // Load sessions and models on mount
  useEffect(() => {
    loadSessions();
    loadModels();
  }, []);

  // When currentModelName changes, select the matching model
  useEffect(() => {
    if (currentModelName && models.length > 0) {
      const matchedModel = models.find(
        (m) =>
          m.ollama_model_name === currentModelName ||
          m.value === currentModelName ||
          m.label === currentModelName,
      );
      if (matchedModel && matchedModel.value !== selectedModel) {
        selectModel(matchedModel.value);
      }
    }
  }, [currentModelName, models, selectedModel, selectModel]);

  const handleSubmit = async () => {
    if (
      input.trim() &&
      !isChatLoading &&
      models.length > 0 &&
      selectedModelData
    ) {
      let message = input.trim();

      if (attachments.length > 0) {
        const attachmentNames = attachments.map((f) => f.name).join(", ");
        message = `${message}\n\n[Attachments: ${attachmentNames}]`;
        clearAttachments();
      }

      resetInput();
      await sendMessage(message);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const toggleMode = () => {
    setMode(mode === "chat" ? "agent" : "chat");
  };

  const handleSelectSession = async (sessionId: number) => {
    await loadSession(sessionId);
    setShowHistory(false);
  };

  const handleDeleteSession = async (sessionId: number) => {
    if (confirm("Delete this chat session?")) {
      await deleteSessionStore(sessionId);
    }
  };

  const handleRenameSession = async (sessionId: number) => {
    const session = sessions.find((s) => s.id === sessionId);
    if (!session) return;

    const newTitle = prompt("Enter new title:", session.title);
    if (newTitle && newTitle.trim()) {
      await updateSessionTitleStore(sessionId, newTitle.trim());
    }
  };

  const handleStartNewChat = () => {
    startNewChatStore();
    startNewChat();
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return "Just now";
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    if (diff < 604800000) return `${Math.floor(diff / 86400000)}d ago`;
    return date.toLocaleDateString();
  };

  const hasValidModels =
    models.length > 0 &&
    models[0].value !== "no-models" &&
    models[0].value !== "error";

  const isSubmitDisabled =
    !input.trim() ||
    isChatLoading ||
    !hasValidModels ||
    !isOllamaReady ||
    !selectedModelData?.ollama_model_name;

  const attachmentCount = attachments.length;
  const hasMessages = currentMessages.length > 0;
  const combinedError = storeError || hookError;

  return (
    <div className="w-full max-w-3xl mx-auto h-full flex flex-col relative">
      {showHistory && (
        <div className="absolute left-0 top-0 bottom-0 w-72 bg-gray-900 border-r border-gray-700 rounded-l-2xl z-10 overflow-y-auto p-4">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-white font-semibold">Chat History</h3>
            <button
              onClick={() => setShowHistory(false)}
              className="text-gray-400 hover:text-white"
            >
              <FiX size={18} />
            </button>
          </div>

          {isLoadingSessions ? (
            <div className="text-gray-400 text-sm">Loading...</div>
          ) : (
            <div className="space-y-2">
              <button
                onClick={() => {
                  handleStartNewChat();
                  setShowHistory(false);
                }}
                className="w-full text-left px-3 py-2 text-sm bg-purple-accent hover:bg-purple-accent/80 text-white rounded-lg transition-colors flex items-center gap-2"
              >
                <FiPlus size={16} />
                New Chat
              </button>

              {sessions.map((session) => (
                <div
                  key={session.id}
                  className={`p-2 rounded-lg cursor-pointer hover:bg-gray-800 transition-colors ${
                    currentSessionId === session.id ? "bg-gray-800" : ""
                  }`}
                >
                  <div
                    className="flex justify-between items-start"
                    onClick={() => handleSelectSession(session.id)}
                  >
                    <div className="flex-1 min-w-0">
                      <div className="text-white text-sm font-medium truncate">
                        {session.title}
                      </div>
                      <div className="text-gray-400 text-xs truncate">
                        {session.model_name}
                      </div>
                      <div className="text-gray-500 text-xs">
                        {formatDate(session.updated_at)}
                      </div>
                    </div>
                    <div className="flex space-x-1 ml-2">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRenameSession(session.id);
                        }}
                        className="text-gray-500 hover:text-gray-300 text-xs p-1"
                      >
                        <FiEdit2 size={12} />
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteSession(session.id);
                        }}
                        className="text-gray-500 hover:text-red-400 text-xs p-1"
                      >
                        <FiTrash2 size={12} />
                      </button>
                    </div>
                  </div>
                </div>
              ))}

              {sessions.length === 0 && (
                <div className="text-gray-500 text-sm text-center mt-4">
                  No chat sessions yet
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {hasMessages ? (
        <>
          <div className="flex-1">
            <ChatMessages
              messages={currentMessages}
              isStreaming={storeIsStreaming}
              modelLoaded={modelLoaded}
              error={combinedError}
              isOllamaReady={isOllamaReady}
            />
          </div>

          <div className="shrink-0 w-full sticky bottom-0 bg-black pb-4">
            {attachmentCount > 0 && (
              <div className="px-4 py-2 text-xs text-white/60 bg-black border-t border-white/5">
                {attachmentCount} file{attachmentCount > 1 ? "s" : ""} attached
                <button
                  onClick={clearAttachments}
                  className="ml-2 text-red-400 hover:text-red-300"
                >
                  Clear
                </button>
              </div>
            )}

            <div className="relative bg-black rounded-2xl border border-white/5 transition-colors">
              <ChatInput
                ref={textareaRef}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                disabled={!hasValidModels || !isOllamaReady}
              />

              <ChatControls
                isSearchEnabled={isSearchEnabled}
                onSearchToggle={() => setIsSearchEnabled(!isSearchEnabled)}
                isCodeEnabled={isCodeEnabled}
                onCodeToggle={() => setIsCodeEnabled(!isCodeEnabled)}
                isAttachmentEnabled={isAttachmentEnabled}
                onAttachmentClick={handleAttachmentClick}
                selectedModel={selectedModel}
                models={models}
                isModelDropdownOpen={isModelDropdownOpen}
                isLoading={isModelsLoading}
                onModelToggle={toggleDropdown}
                onModelSelect={selectModel}
                onModelClose={closeDropdown}
                mode={mode}
                onModeToggle={toggleMode}
                onSubmit={handleSubmit}
                isSubmitDisabled={isSubmitDisabled}
                fileInputRef={fileInputRef}
                onFileChange={handleFileChange}
              />
            </div>

            <div className="flex justify-center mt-3 px-1">
              <div className="text-xs text-white/30">
                Press Enter to send, Shift+Enter for new line
              </div>
            </div>
          </div>
        </>
      ) : (
        <div className="flex flex-col h-full -mt-10">
          <div className="flex-1 flex flex-col items-center justify-center">
            <div className="text-center">
              <h1 className="text-7xl md:text-8xl font-bold tracking-wide font-anton bg-linear-to-r from-purple-accent to-white/80 bg-clip-text text-transparent">
                Solyn
              </h1>
              <p className="text-lg md:text-xl leading-relaxed text-white/80 mt-4">
                A clarity-driven AI for solving complex problems and simplifying
                everyday work.
              </p>
            </div>

            <div className="w-full max-w-3xl mt-8">
              {attachmentCount > 0 && (
                <div className="px-4 py-2 text-xs text-white/60 bg-black border-t border-white/5">
                  {attachmentCount} file{attachmentCount > 1 ? "s" : ""}{" "}
                  attached
                  <button
                    onClick={clearAttachments}
                    className="ml-2 text-red-400 hover:text-red-300"
                  >
                    Clear
                  </button>
                </div>
              )}

              <div className="relative bg-black rounded-2xl border border-white/5 transition-colors">
                <ChatInput
                  ref={textareaRef}
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={handleKeyDown}
                  disabled={!hasValidModels || !isOllamaReady}
                />

                <ChatControls
                  isSearchEnabled={isSearchEnabled}
                  onSearchToggle={() => setIsSearchEnabled(!isSearchEnabled)}
                  isCodeEnabled={isCodeEnabled}
                  onCodeToggle={() => setIsCodeEnabled(!isCodeEnabled)}
                  isAttachmentEnabled={isAttachmentEnabled}
                  onAttachmentClick={handleAttachmentClick}
                  selectedModel={selectedModel}
                  models={models}
                  isModelDropdownOpen={isModelDropdownOpen}
                  isLoading={isModelsLoading}
                  onModelToggle={toggleDropdown}
                  onModelSelect={selectModel}
                  onModelClose={closeDropdown}
                  mode={mode}
                  onModeToggle={toggleMode}
                  onSubmit={handleSubmit}
                  isSubmitDisabled={isSubmitDisabled}
                  fileInputRef={fileInputRef}
                  onFileChange={handleFileChange}
                />
              </div>

              <div className="mt-3 text-xs text-white/30 text-center">
                {!isOllamaReady
                  ? "Ollama is not running. Please install and start Ollama."
                  : !hasValidModels && !isModelsLoading
                    ? "No models installed. Please download a model from the Hugging Face page."
                    : isModelsLoading
                      ? "Loading models..."
                      : !selectedModelData?.ollama_model_name
                        ? "Selected model is not registered with Ollama. Please reinstall."
                        : "Press Enter to send, Shift+Enter for new line"}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
