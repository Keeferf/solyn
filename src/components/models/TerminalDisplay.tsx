import { Terminal, ChevronDown, ChevronUp } from "lucide-react";
import { TerminalOutput } from "./hooks/useOllamaInstallation";
import {
  shouldShowLine,
  getStreamColor,
  getStreamPrefix,
} from "./utils/terminalUtils";

interface TerminalDisplayProps {
  terminalLines: TerminalOutput[];
  isTerminalExpanded: boolean;
  onToggleExpand: () => void;
  terminalEndRef: React.RefObject<HTMLDivElement>;
}

export const TerminalDisplay = ({
  terminalLines,
  isTerminalExpanded,
  onToggleExpand,
  terminalEndRef,
}: TerminalDisplayProps) => {
  const visibleLines = terminalLines.filter((output) =>
    shouldShowLine(output.line),
  );

  return (
    <div className="bg-black/60 border border-white/10 rounded-xl overflow-hidden">
      <div
        className="flex items-center justify-between px-4 py-2 bg-white/5 border-b border-white/10 cursor-pointer hover:bg-white/10 transition-colors"
        onClick={onToggleExpand}
      >
        <div className="flex items-center gap-2">
          <Terminal className="text-white/60" size={18} />
        </div>
        <button
          className="text-white/30 hover:text-white/60 transition-colors cursor-pointer"
          onClick={(e) => {
            e.stopPropagation();
            onToggleExpand();
          }}
        >
          {isTerminalExpanded ? (
            <ChevronUp size={18} />
          ) : (
            <ChevronDown size={18} />
          )}
        </button>
      </div>

      {isTerminalExpanded && (
        <div className="max-h-64 overflow-y-auto p-3 text-left font-mono text-xs">
          {visibleLines.length === 0 ? (
            <p className="text-white/20">Waiting for output...</p>
          ) : (
            visibleLines.map((output, index) => (
              <div
                key={index}
                className={`${getStreamColor(output.stream)} py-0.5 whitespace-pre-wrap break-all`}
              >
                <span className="text-white/20 mr-2 select-none">
                  {getStreamPrefix(output.stream)}
                </span>
                {output.line}
              </div>
            ))
          )}
          <div ref={terminalEndRef} />
        </div>
      )}
    </div>
  );
};
