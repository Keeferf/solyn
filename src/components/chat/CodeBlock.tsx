import { useEffect, useState } from "react";
import { Copy, Check } from "lucide-react";
import { codeToHtml, getSingletonHighlighter } from "shiki";
import { useThemeStore } from "@/stores/themeStore";

interface CodeBlockProps {
  className?: string;
  children: React.ReactNode;
  inline?: boolean;
}

interface RenderedCode {
  html: string;
  bg?: string;
  fg?: string;
}

export const CodeBlock = ({
  className,
  children,
  inline = false,
}: CodeBlockProps) => {
  const [copied, setCopied] = useState(false);
  const [rendered, setRendered] = useState<RenderedCode | null>(null);
  const theme = useThemeStore((state) => state.theme);

  const match = /language-([^\s]+)/.exec(className || "");
  const lang = match ? match[1] : "";
  const codeContent = String(children).replace(/\n$/, "");

  // Highlight on demand: Shiki loads the grammar for this language (and the
  // theme) the first time it is seen, so any bundled language works without
  // a fixed list. The previous render stays until the new one is ready.
  useEffect(() => {
    if (inline) return;
    let active = true;

    (async () => {
      try {
        let html: string;
        try {
          html = await codeToHtml(codeContent, { lang: lang || "text", theme });
        } catch {
          // Unknown language: Shiki throws, so render it as plaintext.
          html = await codeToHtml(codeContent, { lang: "text", theme });
        }
        const { bg, fg } = (
          await getSingletonHighlighter({ themes: [theme] })
        ).getTheme(theme);
        if (active) setRendered({ html, bg, fg });
      } catch {
        // Leave the styled fallback in place.
      }
    })();

    return () => {
      active = false;
    };
  }, [codeContent, lang, theme, inline]);

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

  return (
    <div
      className="shiki-wrapper"
      style={
        rendered
          ? { backgroundColor: rendered.bg, color: rendered.fg }
          : undefined
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
      {rendered ? (
        <div
          className="shiki-container"
          data-language={lang}
          dangerouslySetInnerHTML={{ __html: rendered.html }}
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
