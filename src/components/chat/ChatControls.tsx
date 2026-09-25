import { Search, Terminal, Paperclip, ArrowUp } from "lucide-react";
import { ToggleButton } from "./ToggleButton";
import { ModelSelector } from "./ModelSelector";
import { ModeToggle } from "./ModeToggle";
import { ModelType, ChatModel } from "./hooks/useModelSelection";
import { ModeType } from "./ChatInterface";

interface ChatControlsProps {
  isSearchEnabled: boolean;
  onSearchToggle: () => void;
  isCodeEnabled: boolean;
  onCodeToggle: () => void;
  isAttachmentEnabled: boolean;
  onAttachmentClick: () => void;
  selectedModel: ModelType;
  models: ChatModel[];
  isModelDropdownOpen: boolean;
  isLoading: boolean;
  onModelToggle: () => void;
  onModelSelect: (model: ModelType) => void;
  onModelClose: () => void;
  mode: ModeType;
  onModeToggle: () => void;
  onSubmit: () => void;
  isSubmitDisabled: boolean;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  onFileChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

export const ChatControls = ({
  isSearchEnabled,
  onSearchToggle,
  isCodeEnabled,
  onCodeToggle,
  isAttachmentEnabled,
  onAttachmentClick,
  selectedModel,
  models,
  isModelDropdownOpen,
  isLoading,
  onModelToggle,
  onModelSelect,
  onModelClose,
  mode,
  onModeToggle,
  onSubmit,
  isSubmitDisabled,
  fileInputRef,
  onFileChange,
}: ChatControlsProps) => {
  return (
    <div className="flex items-center justify-between p-2 border-t border-white/5">
      <div className="flex items-center gap-1">
        <ToggleButton
          isActive={isAttachmentEnabled}
          onClick={onAttachmentClick}
          icon={<Paperclip size={18} />}
        />
        <ToggleButton
          isActive={isSearchEnabled}
          onClick={onSearchToggle}
          icon={<Search size={18} />}
        />
        <ToggleButton
          isActive={isCodeEnabled}
          onClick={onCodeToggle}
          icon={<Terminal size={18} />}
        />
        <input
          ref={fileInputRef}
          type="file"
          multiple
          onChange={onFileChange}
          className="hidden"
        />
      </div>

      <div className="flex items-center gap-2">
        <ModelSelector
          selectedModel={selectedModel}
          models={models}
          isOpen={isModelDropdownOpen}
          isLoading={isLoading}
          onToggle={onModelToggle}
          onSelect={onModelSelect}
          onClose={onModelClose}
        />

        <ModeToggle mode={mode} onToggle={onModeToggle} />

        <button
          onClick={onSubmit}
          disabled={isSubmitDisabled}
          className="p-2 bg-purple-accent hover:bg-purple-accent/80 disabled:opacity-40 disabled:cursor-not-allowed rounded-lg transition-colors text-white cursor-pointer"
        >
          <ArrowUp size={18} />
        </button>
      </div>
    </div>
  );
};
