import { CopyButton } from "./CopyButton";

/** A copyable code block. `code` may contain multiple lines. */
export function CodeBlock({ code }: { code: string }) {
  return (
    <div className="codeblock">
      <pre>
        <code>{code}</code>
      </pre>
      <CopyButton value={code} />
    </div>
  );
}
