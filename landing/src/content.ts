// All editable copy for the landing page lives here. Edit this file to update
// wording, links, features, or command lists — the components only render it.

export const GITHUB_URL = "https://github.com/CorrectRoadH/harness-lint";
export const RELEASES_URL = "https://github.com/CorrectRoadH/harness-lint/releases";
export const PLUGINS_URL = "https://github.com/CorrectRoadH/harness-lint/tree/main/plugins";

export const hero = {
  eyebrow: "Lint Driven Development",
  title: "Stop your coding agent from repeating the same mistake.",
  subtitle:
    "When the AI ignores your instructions — even after you write them into AGENTS.md — turn the correction into a fast, strict lint rule. It can't make that mistake again.",
  primaryCta: { label: "Install", href: "#install" },
  secondaryCta: { label: "View on GitHub", href: GITHUB_URL },
};

// The terminal mock shown in the hero. Each line has a kind that drives styling.
export type TermLine = { kind: "cmd" | "out" | "bad" | "good" | "muted"; text: string };

export const heroTerminal: TermLine[] = [
  { kind: "cmd", text: "harness-lint check --changed" },
  { kind: "bad", text: "✗ src/api.ts:42  error  local.no-console-log" },
  { kind: "muted", text: "    console.log(token)  — leaks secrets into logs" },
  { kind: "out", text: "" },
  { kind: "muted", text: "# you corrected this once. now it's a rule." },
  { kind: "cmd", text: "harness-lint check --changed" },
  { kind: "good", text: "✓ No diagnostics. The agent fixed it before you saw it." },
];

export const problem = {
  title: "AGENTS.md is read once, then forgotten.",
  body: "A static block of instructions is buried in context, far from the moment the agent writes the next line. harness-lint runs strict checks on every change and feeds the actual current violations back to the agent — so guidance becomes a guardrail, not a suggestion.",
};

export type Step = { n: string; title: string; body: string };

export const steps: Step[] = [
  {
    n: "01",
    title: "The agent slips",
    body: "It does the thing you've corrected before — a banned API, a leaked secret, the wrong pattern.",
  },
  {
    n: "02",
    title: "Capture the correction",
    body: "Turn the feedback into a small, human-readable GritQL rule. One stable constraint per rule.",
  },
  {
    n: "03",
    title: "Lint enforces it",
    body: "Fast, strict checks run on changed files and report the exact violation, with file and line.",
  },
  {
    n: "04",
    title: "It never recurs",
    body: "The agent sees the diagnostic the moment it writes code and fixes it — every session, automatically.",
  },
];

export type Feature = { title: string; body: string };

export const features: Feature[] = [
  {
    title: "Lint Driven Development",
    body: "Every correction becomes a durable rule. The mistake you fix once is the mistake the agent can never repeat.",
  },
  {
    title: "Human-readable rules",
    body: "Rules are GritQL patterns with Bad / Good examples — easy to read, review, and reason about, unlike opaque lint configs.",
  },
  {
    title: "Built for AI workflows",
    body: "Agent plugins for Claude Code & Codex use lifecycle hooks to feed live violations to the agent right before it writes more code.",
  },
  {
    title: "A rule-pack ecosystem",
    body: "Search, install, and update shared rule packs by language. Pin them in harness.toml and restore them anywhere.",
  },
  {
    title: "Fast and strict",
    body: "Written in Rust with file-level caching. Strict by default, with per-rule levels so warnings and hard failures are explicit.",
  },
  {
    title: "Scoped and configurable",
    body: "File sets, path exceptions, and overrides let you target generated code, silence false positives, and tune levels — without weakening a rule.",
  },
];

export type Command = { cmd: string; note: string };

export const commands: Command[] = [
  { cmd: "harness-lint check --changed", note: "Lint only what you've touched" },
  { cmd: "harness-lint rule list", note: "See every active rule" },
  { cmd: 'harness-lint search "python typing"', note: "Find shareable rule packs" },
  { cmd: "harness-lint install python", note: "Add a rule pack" },
  { cmd: "harness-lint rule verify <id>", note: "Prove a rule actually fires" },
  { cmd: "harness-lint update", note: "Refresh installed packs" },
];

export type InstallStep = { label: string; code: string; note?: string };

export const install: InstallStep[] = [
  {
    label: "1 · Install the CLI",
    code: "brew install getgrit/tap/grit\nbrew install CorrectRoadH/tap/harness-lint",
  },
  {
    label: "2 · Wire it into your repo",
    code: "READ https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/INIT.md and install harness lint for this code repo",
    note: "Paste this to your coding agent — it scaffolds harness.toml and the first rules.",
  },
  {
    label: "3 · Add the agent plugin",
    code: "/plugin marketplace add CorrectRoadH/harness-lint\n/plugin install harness-lint@harness-lint",
    note: "Claude Code. For Codex, swap in: codex plugin marketplace add … / codex plugin add …",
  },
];

export const footerLinks = [
  { label: "GitHub", href: GITHUB_URL },
  { label: "Releases", href: RELEASES_URL },
  { label: "Agent plugins", href: PLUGINS_URL },
];
