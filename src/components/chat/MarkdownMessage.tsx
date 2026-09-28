import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import rehypeSanitize from "rehype-sanitize";
import rehypeSlug from "rehype-slug";
import "katex/dist/katex.min.css";
import { CodeBlock } from "./CodeBlock";
import { useHighlighter } from "./hooks/useHighlighter";

interface MarkdownMessageProps {
  content: string;
  isUser?: boolean;
}

export const MarkdownMessage = ({
  content,
  isUser = false,
}: MarkdownMessageProps) => {
  const loaded = useHighlighter();

  if (isUser) {
    return <div className="text-sm whitespace-pre-wrap">{content}</div>;
  }

  return (
    <div className="markdown-content prose prose-invert max-w-none">
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkMath]}
        rehypePlugins={[
          rehypeKatex,
          [
            rehypeSanitize,
            {
              attributes: {
                "*": ["style", "className", "class", "data-*"],
                span: ["style", "class"],
                pre: ["style", "class"],
                code: ["style", "class"],
              },
              strip: [],
            },
          ],
          rehypeSlug,
        ]}
        components={{
          code({ className, children, ...props }: any) {
            const inline = props.inline || false;

            return (
              <CodeBlock
                className={className}
                highlighter={loaded?.highlighter ?? null}
                theme={loaded?.theme ?? null}
                inline={inline}
                {...props}
              >
                {children}
              </CodeBlock>
            );
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};
