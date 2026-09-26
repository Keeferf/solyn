import { useEffect, useRef } from "react";
import { ChatMessage } from "./hooks/useChat";
import { MarkdownMessage } from "./MarkdownMessage";
import { RingSpinner } from "../ui/RingSpinner";

interface ChatMessagesProps {
  messages: ChatMessage[];
  isStreaming: boolean;
  modelLoaded: boolean;
  error: string | null;
  isOllamaReady: boolean;
}

export const ChatMessages = ({
  messages,
  isStreaming,
  modelLoaded,
  error,
  isOllamaReady: _isOllamaReady,
}: ChatMessagesProps) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const autoScrollRef = useRef(true);
  const prevCountRef = useRef(messages.length);
  const hasMessages = messages.length > 0;

  // Follow the bottom only while the user is already near it
  useEffect(() => {
    const end = messagesEndRef.current;
    if (!end) return;

    let container: HTMLElement | null = end.parentElement;
    while (container) {
      const overflowY = getComputedStyle(container).overflowY;
      if (overflowY === "auto" || overflowY === "scroll") break;
      container = container.parentElement;
    }
    const scrollParent = container;
    if (!scrollParent) return;

    const onScroll = () => {
      autoScrollRef.current =
        scrollParent.scrollHeight -
          scrollParent.scrollTop -
          scrollParent.clientHeight <
        120;
    };
    onScroll();
    scrollParent.addEventListener("scroll", onScroll, { passive: true });
    return () => scrollParent.removeEventListener("scroll", onScroll);
  }, [hasMessages]);

  useEffect(() => {
    const grew = messages.length > prevCountRef.current;
    prevCountRef.current = messages.length;
    if (!autoScrollRef.current && !grew) return;
    if (grew) autoScrollRef.current = true;
    messagesEndRef.current?.scrollIntoView({
      behavior: isStreaming ? "auto" : "smooth",
    });
  }, [messages, isStreaming]);

  if (!hasMessages) {
    return null;
  }

  return (
    <div className="px-4 py-6 space-y-6">
      {messages.map((message, index) => {
        const isEmptyAssistant =
          index === messages.length - 1 &&
          message.role === "assistant" &&
          message.content === "";

        const isUser = message.role === "user";
        const hasThinking = !isUser && !!message.thinking;

        return (
          <div
            key={index}
            className={`flex ${isUser ? "justify-end" : "justify-start"}`}
          >
            <div
              className={`${
                isUser
                  ? "max-w-[85%] rounded-2xl px-4 py-3 bg-purple-accent text-white"
                  : "w-full max-w-4xl"
              }`}
            >
              {isUser ? (
                <div className="text-sm whitespace-pre-wrap">
                  {message.content}
                </div>
              ) : (
                <div>
                  {hasThinking && (
                    <details
                      open={message.content === ""}
                      className="mb-3 rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white/50"
                    >
                      <summary className="cursor-pointer select-none text-white/40 hover:text-white/70">
                        <span className="inline-flex items-center gap-2">
                          Thinking
                          {isEmptyAssistant && isStreaming && (
                            <RingSpinner size={12} />
                          )}
                        </span>
                      </summary>
                      <div className="mt-2 whitespace-pre-wrap break-words">
                        {message.thinking}
                      </div>
                    </details>
                  )}
                  {isEmptyAssistant && isStreaming && !hasThinking ? (
                    <div className="flex items-center gap-2 text-sm text-white/50">
                      <RingSpinner size={12} />
                      {modelLoaded ? "Thinking" : "Loading model"}
                    </div>
                  ) : (
                    <MarkdownMessage
                      content={message.content}
                      isUser={isUser}
                    />
                  )}
                </div>
              )}
            </div>
          </div>
        );
      })}

      {error && (
        <div className="flex justify-center">
          <div className="bg-error-bg text-error border border-error-border rounded-lg px-4 py-2 text-sm">
            Error: {error}
          </div>
        </div>
      )}

      <div ref={messagesEndRef} />
    </div>
  );
};
