import { useEffect, useRef, useState } from "react";
import { Check, Search, Settings, X } from "lucide-react";
import {
  AVAILABLE_THEMES,
  getThemeType,
  useThemeStore,
} from "@/stores/themeStore";
import { CodeBlock } from "@/components/chat/CodeBlock";

const PREVIEW_CODE = `// Theme preview
import { useState } from "react";

interface User {
  name: string;
  age: number;
}

export function Greeting({ user }: { user: User }) {
  const [count, setCount] = useState(0);
  return (
    <button onClick={() => setCount(count + 1)}>
      Hello, {user.name}! Clicked {count} times
    </button>
  );
}`;

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const ThemePreview = () => (
  <CodeBlock className="language-tsx">{PREVIEW_CODE}</CodeBlock>
);

export const SettingsModal = ({ isOpen, onClose }: SettingsModalProps) => {
  const modalRef = useRef<HTMLDivElement>(null);
  const { theme, setTheme } = useThemeStore();
  const [query, setQuery] = useState("");

  const filtered = AVAILABLE_THEMES.filter((t) =>
    t.label.toLowerCase().includes(query.trim().toLowerCase()),
  );
  const groups = [
    { label: "Dark", themes: filtered.filter((t) => getThemeType(t.id) === "dark") },
    { label: "Light", themes: filtered.filter((t) => getThemeType(t.id) === "light") },
  ].filter((g) => g.themes.length > 0);

  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (event: MouseEvent) => {
      if (
        modalRef.current &&
        !modalRef.current.contains(event.target as Node)
      ) {
        onClose();
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);

    document.body.style.overflow = "hidden";

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
      document.body.style.overflow = "unset";
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-100">
      <div
        ref={modalRef}
        className="bg-black border border-white/10 rounded-xl w-[95vw] h-[90vh] max-w-none p-6 shadow-2xl animate-fadeIn animate-slideUp flex flex-col"
      >
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-full bg-white/5">
              <Settings className="w-5 h-5 text-white/70" />
            </div>
            <h2 className="text-xl font-semibold text-white">Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="text-white/40 hover:text-white/70 transition-colors cursor-pointer"
            aria-label="Close settings"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex-1 min-h-0 rounded-xl border border-white/10 p-4 flex flex-col md:flex-none">
          <div className="flex-1 min-h-0 grid gap-6 grid-rows-2 md:flex-none md:grid-rows-none md:grid-cols-2">
            <div className="relative min-h-0">
              <div className="absolute inset-0 flex flex-col">
                <span className="text-xs font-medium text-white/40 uppercase tracking-wider">
                  Theme
                </span>
                <div className="relative mt-3">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/30 pointer-events-none" />
                  <input
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder="Search themes"
                    className="w-full rounded-lg bg-white/5 border border-white/10 pl-9 pr-3 py-2 text-sm text-white placeholder-white/30 focus:outline-none focus:border-white/20"
                  />
                </div>
                <div className="mt-2 flex-1 min-h-0 overflow-y-auto bg-white/5 rounded-lg p-1">
                  {groups.length === 0 && (
                    <p className="px-3 py-2 text-sm text-white/40">
                      No themes found
                    </p>
                  )}
                  {groups.map((group) => (
                    <div key={group.label}>
                      <p className="px-3 pt-2 pb-1 text-[10px] font-semibold uppercase tracking-wider text-white/30">
                        {group.label}
                      </p>
                      {group.themes.map((t) => (
                        <button
                          key={t.id}
                          onClick={() => setTheme(t.id)}
                          className="w-full flex items-center justify-between gap-3 px-3 py-2 text-sm text-white/80 hover:bg-white/5 rounded transition-colors cursor-pointer"
                        >
                          <span className="truncate">{t.label}</span>
                          {theme === t.id && (
                            <Check
                              size={14}
                              className="text-purple-accent shrink-0"
                            />
                          )}
                        </button>
                      ))}
                    </div>
                  ))}
                </div>
              </div>
            </div>

            <div className="min-h-0 flex flex-col">
              <span className="text-xs font-medium text-white/40 uppercase tracking-wider">
                Preview
              </span>
              <div className="mt-3 min-h-0 [&_.shiki-wrapper]:my-0">
                <ThemePreview />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
