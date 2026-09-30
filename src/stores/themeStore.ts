import { create } from "zustand";
import { persist } from "zustand/middleware";
import { bundledThemesInfo } from "shiki";

const TYPE_BY_ID = new Map(bundledThemesInfo.map((t) => [t.id, t.type]));

export const getThemeType = (id: string): "light" | "dark" =>
  TYPE_BY_ID.get(id) ?? "dark";

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

interface ThemeState {
  theme: string;
  setTheme: (theme: string) => void;
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      theme: "github-dark",
      setTheme: (theme) => set({ theme }),
    }),
    {
      name: "theme-storage",
    },
  ),
);
