import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import rehypeSanitize from "rehype-sanitize";
import rehypeSlug from "rehype-slug";
import "katex/dist/katex.min.css";
import { CodeBlock } from "./CodeBlock";

interface MarkdownMessageProps {
  content: string;
  isUser?: boolean;
}

export const MarkdownMessage = ({
  content,
  isUser = false,
}: MarkdownMessageProps) => {
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
          code({ className, children }: any) {
            // react-markdown v10 dropped the `inline` flag. Inline code
            // spans never contain a newline, while fenced blocks keep the
            // trailing newline from the source.
            const inline =
              children !== undefined && !String(children).includes("\n");

            return (
              <CodeBlock className={className} inline={inline}>
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
