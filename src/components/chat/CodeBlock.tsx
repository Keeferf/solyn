import { useState, useMemo } from "react";
import { FiCopy, FiCheck } from "react-icons/fi";
import { type Highlighter } from "shiki";
import { useThemeStore } from "@/stores/themeStore";
import { getThemeColors } from "@/utils/themeColors";

interface CodeBlockProps {
  className?: string;
  children: React.ReactNode;
  highlighter: Highlighter | null;
  inline?: boolean;
}

export const CodeBlock = ({
  className,
  children,
  highlighter,
  inline = false,
}: CodeBlockProps) => {
  const [copied, setCopied] = useState(false);
  const { theme } = useThemeStore();

  // Get theme-aware colors
  const themeColors = useMemo(() => getThemeColors(theme), [theme]);

  const match = /language-(\w+)/.exec(className || "");
  const lang = match ? match[1] : "";
  const codeContent = String(children).replace(/\n$/, "");

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(codeContent);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {}
  };

  // If it's inline code
  if (inline || !highlighter || !lang) {
    return inline ? (
      <code className={className}>{children}</code>
    ) : (
      <div
        className="shiki-wrapper"
        style={{
          backgroundColor: themeColors.background,
          borderColor: themeColors.border,
        }}
      >
        <div
          className="shiki-header"
          style={{
            backgroundColor: themeColors.headerBackground,
            borderBottomColor: themeColors.border,
          }}
        >
          <span
            className="shiki-language"
            style={{ color: themeColors.languageLabel }}
          >
            {lang || "code"}
          </span>
          <button
            onClick={handleCopy}
            className="shiki-copy-button"
            style={{ color: themeColors.copyButton }}
            aria-label="Copy code"
            onMouseEnter={(e) => {
              e.currentTarget.style.color = themeColors.copyButtonHover;
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = themeColors.copyButton;
            }}
          >
            {copied ? (
              <>
                <FiCheck size={14} />
                <span>Copied!</span>
              </>
            ) : (
              <>
                <FiCopy size={14} />
                <span>Copy</span>
              </>
            )}
          </button>
        </div>
        <div
          className="shiki-container shiki-container-fallback"
          style={{ backgroundColor: themeColors.background }}
        >
          <pre className={className}>
            <code className={className}>{children}</code>
          </pre>
        </div>
      </div>
    );
  }

  try {
    // Use Shiki to highlight the code with the selected theme
    const html = highlighter.codeToHtml(codeContent, {
      lang,
      theme: theme,
    });

    return (
      <div
        className="shiki-wrapper"
        style={{
          backgroundColor: themeColors.background,
          borderColor: themeColors.border,
        }}
      >
        <div
          className="shiki-header"
          style={{
            backgroundColor: themeColors.headerBackground,
            borderBottomColor: themeColors.border,
          }}
        >
          <span
            className="shiki-language"
            style={{ color: themeColors.languageLabel }}
          >
            {lang}
          </span>
          <button
            onClick={handleCopy}
            className="shiki-copy-button"
            style={{ color: themeColors.copyButton }}
            aria-label="Copy code"
            onMouseEnter={(e) => {
              e.currentTarget.style.color = themeColors.copyButtonHover;
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = themeColors.copyButton;
            }}
          >
            {copied ? (
              <>
                <FiCheck size={14} />
                <span>Copied!</span>
              </>
            ) : (
              <>
                <FiCopy size={14} />
                <span>Copy</span>
              </>
            )}
          </button>
        </div>
        <div
          className="shiki-container"
          data-language={lang}
          style={{ backgroundColor: themeColors.background }}
          dangerouslySetInnerHTML={{ __html: html }}
        />
      </div>
    );
  } catch (error) {
    return (
      <div
        className="shiki-wrapper"
        style={{
          backgroundColor: themeColors.background,
          borderColor: themeColors.border,
        }}
      >
        <div
          className="shiki-header"
          style={{
            backgroundColor: themeColors.headerBackground,
            borderBottomColor: themeColors.border,
          }}
        >
          <span
            className="shiki-language"
            style={{ color: themeColors.languageLabel }}
          >
            {lang}
          </span>
          <button
            onClick={handleCopy}
            className="shiki-copy-button"
            style={{ color: themeColors.copyButton }}
            aria-label="Copy code"
            onMouseEnter={(e) => {
              e.currentTarget.style.color = themeColors.copyButtonHover;
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = themeColors.copyButton;
            }}
          >
            {copied ? (
              <>
                <FiCheck size={14} />
                <span>Copied!</span>
              </>
            ) : (
              <>
                <FiCopy size={14} />
                <span>Copy</span>
              </>
            )}
          </button>
        </div>
        <div
          className="shiki-container shiki-container-fallback"
          style={{ backgroundColor: themeColors.background }}
        >
          <pre className={className}>
            <code className={className}>{children}</code>
          </pre>
        </div>
      </div>
    );
  }
};
