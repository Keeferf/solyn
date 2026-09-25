import { Cpu } from "lucide-react";
import { GGUFFile } from "../hooks/useHuggingFaceModels";
import {
  formatFileSize,
  formatParameterCount,
  getQuantizationDescription,
} from "../utils/modalUtils";

interface QuantizationCardProps {
  quant: string;
  files: GGUFFile[];
  isSelected: boolean;
  onSelect: (file: GGUFFile) => void;
}

export const QuantizationCard = ({
  quant,
  files,
  isSelected,
  onSelect,
}: QuantizationCardProps) => {
  const largestFile = files.reduce((a, b) => (a.size > b.size ? a : b));
  const paramCount = largestFile.parameter_count;

  return (
    <div
      className={`p-4 bg-black rounded-lg border-2 transition-all cursor-pointer hover:bg-white/5 ${
        isSelected
          ? "border-purple-accent bg-purple-accent/10"
          : "border-white/10 hover:border-white/20"
      }`}
      onClick={() => onSelect(largestFile)}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-white font-medium text-sm">{quant}</span>
            {isSelected && (
              <span className="text-purple-accent text-xs">▼ Selected</span>
            )}
          </div>
          <p className="text-white/40 text-xs mt-1 line-clamp-1">
            {getQuantizationDescription(quant)}
          </p>
          {paramCount && (
            <div className="flex items-center gap-1 mt-1 text-emerald-400 text-xs">
              <Cpu size={12} />
              <span>{formatParameterCount(paramCount)}</span>
            </div>
          )}
        </div>
        <div className="text-right shrink-0 ml-2">
          <div className="text-white/60 text-sm font-mono">
            {formatFileSize(largestFile.size)}
          </div>
          {files.length > 1 && (
            <div className="text-white/30 text-xs">
              +{files.length - 1} more
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
