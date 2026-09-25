// src/components/history/ChatItem.tsx
import { useState, useRef } from "react";
import { Check } from "lucide-react";
import type { ChatSession } from "./types";
import { ChatDate } from "./ChatDate";
import { DropdownMenu } from "./DropdownMenu";

interface ChatItemProps {
  chat: ChatSession;
  preview: string;
  onSelect: (chatId: number) => void;
  onPin: (chatId: number) => void;
  onRename: (chatId: number) => void;
  onDelete: (chatId: number) => void;
  isSelectionMode?: boolean;
  isSelected?: boolean;
  isActive?: boolean;
  onToggleSelect?: (chatId: number) => void;
}

export const ChatItem = ({
  chat,
  preview,
  onSelect,
  onPin,
  onRename,
  onDelete,
  isSelectionMode = false,
  isSelected = false,
  isActive = false,
  onToggleSelect,
}: ChatItemProps) => {
  const [isHovered, setIsHovered] = useState(false);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const ellipsisRef = useRef<HTMLSpanElement>(null);

  const handleEllipsisClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsDropdownOpen(!isDropdownOpen);
  };

  const handleChatClick = () => {
    if (isSelectionMode && onToggleSelect) {
      onToggleSelect(chat.id);
      return;
    }

    if (isDropdownOpen) {
      setIsDropdownOpen(false);
      return;
    }
    onSelect(chat.id);
  };

  const handleCheckboxClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onToggleSelect) {
      onToggleSelect(chat.id);
    }
  };

  // Truncate preview if too long
  const truncatedPreview =
    preview.length > 60 ? preview.slice(0, 60) + "..." : preview;

  return (
    <div
      onClick={handleChatClick}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      className={`
        group relative transition-all duration-200 cursor-pointer
        rounded-lg px-4 py-3 mx-4 mb-2
        ${isSelected ? "bg-purple-accent/20 border border-purple-accent/30" : ""}
        ${isActive ? "bg-purple-accent/10 border border-purple-accent/20" : ""}
        ${isHovered && !isSelected && !isActive ? "bg-white/10" : ""}
        ${!isHovered && !isSelected && !isActive ? "bg-white/5" : ""}
      `}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center flex-1 min-w-0 gap-3">
          {isSelectionMode && (
            <div
              onClick={handleCheckboxClick}
              className="shrink-0 w-5 h-5 rounded border-2 flex items-center justify-center cursor-pointer transition-colors"
              style={{
                borderColor: isSelected
                  ? "var(--color-purple-accent)"
                  : "rgba(255,255,255,0.3)",
                backgroundColor: isSelected
                  ? "var(--color-purple-accent)"
                  : "transparent",
              }}
            >
              {isSelected && <Check className="w-3 h-3 text-white" />}
            </div>
          )}
          <div className="flex-1 min-w-0">
            <p
              className={`font-medium text-base truncate ${isActive ? "text-purple-accent" : "text-white"}`}
            >
              {truncatedPreview || "New Chat"}
            </p>
            <div className="flex items-center gap-3 text-xs text-white/30 mt-0.5">
              <span>Model: {chat.model_name}</span>
              {isActive && (
                <span className="text-purple-accent/60">● Active</span>
              )}
            </div>
          </div>
        </div>

        {!isSelectionMode && (
          <div className="flex items-center h-full shrink-0 relative ml-3">
            <div className="flex flex-col text-xs text-white/30 min-w-17.5">
              <ChatDate
                createdAt={chat.created_at}
                showEllipsis={isHovered}
                onEllipsisClick={handleEllipsisClick}
              />
            </div>

            <DropdownMenu
              isOpen={isDropdownOpen}
              onClose={() => setIsDropdownOpen(false)}
              onPin={() => onPin(chat.id)}
              onRename={() => onRename(chat.id)}
              onDelete={() => onDelete(chat.id)}
              anchorRef={ellipsisRef}
            />
          </div>
        )}
      </div>
    </div>
  );
};
