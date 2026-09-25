import { useState, useEffect, useRef } from "react";
import { PenTool, Check, ChevronDown } from "lucide-react";
import { useThemeStore } from "@/stores/themeStore";

// All available themes from Shiki
export const AVAILABLE_THEMES = [
  { id: "github-dark", label: "GitHub Dark" },
  { id: "dark-plus", label: "Dark Plus" },
  { id: "one-dark-pro", label: "One Dark Pro" },
  { id: "dracula", label: "Dracula" },
  { id: "dracula-soft", label: "Dracula Soft" },
  { id: "nord", label: "Nord" },
  { id: "monokai", label: "Monokai" },
  { id: "material-theme", label: "Material Theme" },
  { id: "material-theme-darker", label: "Material Darker" },
  { id: "material-theme-ocean", label: "Material Ocean" },
  { id: "material-theme-palenight", label: "Material Palenight" },
  { id: "slack-dark", label: "Slack Dark" },
  { id: "vitesse-dark", label: "Vitesse Dark" },
  { id: "tokyo-night", label: "Tokyo Night" },
  { id: "catppuccin-mocha", label: "Catppuccin Mocha" },
  { id: "catppuccin-macchiato", label: "Catppuccin Macchiato" },
  { id: "catppuccin-frappe", label: "Catppuccin Frappé" },
  { id: "everforest-dark", label: "Everforest Dark" },
  { id: "gruvbox-dark-medium", label: "Gruvbox Dark" },
  { id: "kanagawa-wave", label: "Kanagawa Wave" },
  { id: "night-owl", label: "Night Owl" },
  { id: "rose-pine", label: "Rosé Pine" },
  { id: "rose-pine-moon", label: "Rosé Pine Moon" },
  { id: "synthwave-84", label: "Synthwave '84" },
  { id: "laserwave", label: "LaserWave" },
  { id: "aurora-x", label: "Aurora X" },
  { id: "houston", label: "Houston" },
  { id: "vesper", label: "Vesper" },
  { id: "red", label: "Red" },
  { id: "poimandres", label: "Poimandres" },
  { id: "min-dark", label: "Min Dark" },
  { id: "github-dark-dimmed", label: "GitHub Dark Dimmed" },
  { id: "github-dark-high-contrast", label: "GitHub Dark High Contrast" },
  { id: "github-light", label: "GitHub Light" },
  { id: "light-plus", label: "Light Plus" },
  { id: "one-light", label: "One Light" },
];

interface ThemeSwitcherProps {
  collapsed?: boolean;
}

export const ThemeSwitcher = ({ collapsed = false }: ThemeSwitcherProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { theme, setTheme } = useThemeStore();

  // Find current theme label
  const currentTheme = AVAILABLE_THEMES.find((t) => t.id === theme);
  const currentLabel = currentTheme?.label || "Theme";

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Close dropdown on escape key
  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    };

    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, []);

  const handleThemeSelect = (themeId: string) => {
    setTheme(themeId);
    setIsOpen(false);
  };

  if (collapsed) {
    return (
      <div className="relative" ref={dropdownRef}>
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="w-full flex items-center justify-center rounded-lg transition-all duration-200 text-white/60 hover:bg-white/5 hover:text-white cursor-pointer p-2"
          title="Change theme"
        >
          <PenTool size={20} />
        </button>

        {isOpen && (
          <div className="absolute right-0 top-full mt-2 w-56 max-h-80 overflow-y-auto bg-[#1e1e2e] border border-white/10 rounded-lg shadow-xl z-50">
            <div className="sticky top-0 bg-[#1e1e2e] px-3 py-2 border-b border-white/10 z-10">
              <span className="text-xs font-medium text-white/40 uppercase tracking-wider">
                Select Theme
              </span>
            </div>
            <div className="p-1">
              {AVAILABLE_THEMES.map((t) => (
                <button
                  key={t.id}
                  onClick={() => handleThemeSelect(t.id)}
                  className="w-full flex items-center justify-between px-3 py-2 text-sm text-white/80 hover:bg-white/5 rounded transition-colors"
                >
                  <span>{t.label}</span>
                  {theme === t.id && (
                    <Check size={14} className="text-purple-accent" />
                  )}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between rounded-lg transition-all duration-200 text-white/60 hover:bg-white/5 hover:text-white cursor-pointer px-3 py-2"
      >
        <div className="flex items-center gap-3">
          <PenTool size={18} />
          <span className="text-sm font-medium">Theme</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-white/40">{currentLabel}</span>
          <ChevronDown
            size={14}
            className={`transition-transform duration-200 ${
              isOpen ? "rotate-180" : ""
            }`}
          />
        </div>
      </button>

      {isOpen && (
        <div className="absolute left-0 top-full mt-2 w-full max-h-80 overflow-y-auto bg-[#1e1e2e] border border-white/10 rounded-lg shadow-xl z-50">
          <div className="sticky top-0 bg-[#1e1e2e] px-3 py-2 border-b border-white/10 z-10">
            <span className="text-xs font-medium text-white/40 uppercase tracking-wider">
              Select Theme
            </span>
          </div>
          <div className="p-1">
            {AVAILABLE_THEMES.map((t) => (
              <button
                key={t.id}
                onClick={() => handleThemeSelect(t.id)}
                className="w-full flex items-center justify-between px-3 py-2 text-sm text-white/80 hover:bg-white/5 rounded transition-colors"
              >
                <span>{t.label}</span>
                {theme === t.id && (
                  <Check size={14} className="text-purple-accent" />
                )}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
