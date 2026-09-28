import { useState } from "react";
import { Copy, Check } from "lucide-react";
import { type Highlighter } from "shiki";

interface CodeBlockProps {
  className?: string;
  children: React.ReactNode;
  highlighter: Highlighter | null;
  theme: string | null;
  inline?: boolean;
}

export const CodeBlock = ({
  className,
  children,
  highlighter,
  theme,
  inline = false,
}: CodeBlockProps) => {
  const [copied, setCopied] = useState(false);

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

  if (inline) {
    return <code className={className}>{children}</code>;
  }

  // Use the resolved Shiki theme for the chrome so it always matches the
  // token colors. The highlighter only hands us a theme once it's loaded.
  let colors: { bg?: string; fg?: string } | null = null;
  let highlighted: string | null = null;
  if (highlighter && theme) {
    try {
      colors = highlighter.getTheme(theme);
      highlighted = highlighter.codeToHtml(codeContent, {
        lang: highlighter.getLoadedLanguages().includes(lang) ? lang : "text",
        theme,
      });
    } catch {
      colors = null;
      highlighted = null;
    }
  }

  return (
    <div
      className="shiki-wrapper"
      style={
        colors ? { backgroundColor: colors.bg, color: colors.fg } : undefined
      }
    >
      <div className="shiki-header">
        <span className="shiki-language">{lang || "code"}</span>
        <button
          onClick={handleCopy}
          className="shiki-copy-button"
          aria-label="Copy code"
        >
          {copied ? (
            <>
              <Check size={14} />
              <span>Copied!</span>
            </>
          ) : (
            <>
              <Copy size={14} />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>
      {highlighted ? (
        <div
          className="shiki-container"
          data-language={lang}
          dangerouslySetInnerHTML={{ __html: highlighted }}
        />
      ) : (
        <div className="shiki-container shiki-container-fallback">
          <pre className={className}>
            <code className={className}>{children}</code>
          </pre>
        </div>
      )}
    </div>
  );
};
