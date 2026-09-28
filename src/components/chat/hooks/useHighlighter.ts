import { useEffect, useState } from "react";
import { getSingletonHighlighter, type Highlighter } from "shiki";
import { useThemeStore } from "@/stores/themeStore";

const SUPPORTED_LANGUAGES = [
  "javascript",
  "typescript",
  "python",
  "rust",
  "go",
  "html",
  "css",
  "json",
  "yaml",
  "sql",
  "bash",
  "shell",
  "markdown",
  "xml",
  "c",
  "cpp",
  "csharp",
  "java",
  "ruby",
  "php",
  "swift",
  "kotlin",
  "dart",
  "r",
  "scala",
  "perl",
  "lua",
  "groovy",
  "powershell",
  "dockerfile",
  "nginx",
  "graphql",
];

export interface LoadedHighlighter {
  highlighter: Highlighter;
  theme: string;
}

/**
 * Shared Shiki highlighter. All code blocks reuse one instance, and the
 * previously loaded theme is kept until the next one finishes loading so
 * blocks never flash to an unstyled state when the theme changes.
 */
export const useHighlighter = (): LoadedHighlighter | null => {
  const theme = useThemeStore((state) => state.theme);
  const [loaded, setLoaded] = useState<LoadedHighlighter | null>(null);

  useEffect(() => {
    let active = true;

    getSingletonHighlighter({
      themes: [theme],
      langs: SUPPORTED_LANGUAGES,
    }).then((highlighter) => {
      if (active) setLoaded({ highlighter, theme });
    });

    return () => {
      active = false;
    };
  }, [theme]);

  return loaded;
};
