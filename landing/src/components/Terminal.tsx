import type { TermLine } from "../content";

/** A static, decorative terminal window with traffic-light dots. */
export function Terminal({ title, lines }: { title: string; lines: TermLine[] }) {
  return (
    <div className="terminal" role="img" aria-label="harness-lint terminal demo">
      <div className="terminal-bar">
        <span className="dot dot-red" />
        <span className="dot dot-amber" />
        <span className="dot dot-green" />
        <span className="terminal-title">{title}</span>
      </div>
      <pre className="terminal-body">
        {lines.map((line, i) =>
          line.text === "" ? (
            <span key={i} className="term-line">
              {" "}
            </span>
          ) : (
            <span key={i} className={`term-line term-${line.kind}`}>
              {line.kind === "cmd" && <span className="term-prompt">$ </span>}
              {line.text}
            </span>
          ),
        )}
      </pre>
    </div>
  );
}
