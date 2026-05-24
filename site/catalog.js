const packs = [
  {
    id: "python",
    title: "Python Application Conventions",
    languages: ["python"],
    description: "Typed Python runtime rules for explicit boundaries, safer dynamic access, and common debug hazards.",
    rules: [
      "Avoid getattr in normal application flow",
      "Avoid broad object types",
      "Avoid committed print debugging",
      "Avoid eval and exec",
      "Avoid assert for runtime validation",
    ],
    install: "harness-lint pack add python github:CorrectRoadH/harness-lint@main#packs/python",
    keywords: ["python", "pydantic", "typed", "getattr", "object", "logging"],
  },
  {
    id: "go",
    title: "Go Service Conventions",
    languages: ["go"],
    description: "Small Go rules for production service hygiene: context flow, panics, and debug output.",
    rules: [
      "Avoid context.TODO in application flow",
      "Avoid panic in normal service flow",
      "Avoid fmt print debugging",
      "Avoid process exits in service flow",
    ],
    install: "harness-lint pack add go github:CorrectRoadH/harness-lint@main#packs/go",
    keywords: ["go", "golang", "context", "panic", "logging"],
  },
  {
    id: "typescript",
    title: "TypeScript Application Conventions",
    languages: ["typescript", "javascript"],
    description: "TypeScript rules for safer application code: no committed console debugging, no var, and fewer untyped escape hatches.",
    rules: [
      "Avoid committed console.log",
      "Avoid var declarations",
      "Avoid explicit any",
      "Avoid committed debugger statements",
    ],
    install: "harness-lint pack add typescript github:CorrectRoadH/harness-lint@main#packs/typescript",
    keywords: ["typescript", "javascript", "react", "next", "console", "any"],
  },
];

const template = document.querySelector("#pack-template");
const catalog = document.querySelector("#catalog");
const search = document.querySelector("#search");

function render(query = "") {
  const normalized = query.trim().toLowerCase();
  catalog.replaceChildren();
  const matches = packs.filter((pack) => {
    const text = [pack.id, pack.title, pack.description, ...pack.languages, ...pack.rules, ...pack.keywords]
      .join(" ")
      .toLowerCase();
    return !normalized || text.includes(normalized);
  });

  for (const pack of matches) {
    const node = template.content.cloneNode(true);
    node.querySelector(".language").textContent = pack.languages.join(" / ");
    node.querySelector("h2").textContent = pack.title;
    node.querySelector(".description").textContent = pack.description;
    const list = node.querySelector("ul");
    for (const rule of pack.rules) {
      const item = document.createElement("li");
      item.textContent = rule;
      list.append(item);
    }
    node.querySelector(".install").textContent = pack.install;
    catalog.append(node);
  }
}

search.addEventListener("input", () => render(search.value));
render();
