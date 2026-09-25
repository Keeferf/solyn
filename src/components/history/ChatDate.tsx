import { Clock } from "lucide-react";

interface ChatDateProps {
  createdAt: string;
  showEllipsis?: boolean;
  onEllipsisClick?: (e: React.MouseEvent) => void;
}

export const ChatDate = ({
  createdAt,
  showEllipsis = false,
  onEllipsisClick,
}: ChatDateProps) => {
  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
    });
  };

  if (showEllipsis) {
    return (
      <div className="flex items-center justify-center text-xs text-white/30 w-full">
        <span
          onClick={onEllipsisClick}
          className="px-2 py-1 rounded border border-white/30 hover:border-purple-accent/60 transition-colors duration-200 cursor-pointer hover:bg-white/5"
        >
          •••
        </span>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-center gap-1 text-xs text-white/30 w-full">
      <Clock className="w-3 h-3" />
      <span>{formatDate(createdAt)}</span>
    </div>
  );
};
