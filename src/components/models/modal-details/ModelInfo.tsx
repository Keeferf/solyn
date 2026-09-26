import { User, CloudDownload, Heart } from "lucide-react";
import { formatDownloads } from "../utils/modalUtils";

export const ModelInfo = ({
  details,
}: {
  details: {
    name?: string;
    model_id: string;
    author?: string;
    downloads?: number;
    likes?: number;
  };
}) => (
  <div className="flex-1 min-w-0">
    <h3 className="text-xl font-bold text-white truncate">
      {details.name || details.model_id}
    </h3>
    <div className="flex items-center gap-3 mt-1">
      {details.author && (
        <div className="flex items-center gap-1 text-white/40 text-sm">
          <User size={14} />
          <span>{details.author}</span>
        </div>
      )}
      {details.downloads !== undefined && details.downloads > 0 && (
        <div className="flex items-center gap-1 text-white/40 text-sm">
          <CloudDownload size={14} />
          <span>{formatDownloads(details.downloads)}</span>
        </div>
      )}
      {details.likes !== undefined && details.likes > 0 && (
        <div className="flex items-center gap-1 text-white/40 text-sm">
          <Heart size={14} />
          <span>{details.likes}</span>
        </div>
      )}
    </div>
  </div>
);
