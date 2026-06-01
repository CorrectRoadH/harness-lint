const template = document.querySelector("#pack-template");
const catalog = document.querySelector("#catalog");
const search = document.querySelector("#search");

let packs = [];

async function loadCatalog() {
  const response = await fetch("./catalog.json", { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`catalog.json returned ${response.status}`);
  }
  packs = await response.json();
}

function render(query = "") {
  const normalized = query.trim().toLowerCase();
  catalog.replaceChildren();
  const matches = packs.filter((pack) => {
    const text = [
      pack.id,
      pack.title,
      pack.description,
      ...(pack.languages || []),
      ...(pack.keywords || []),
      ...(pack.rules || []).map((rule) => `${rule.title} ${rule.reason}`),
    ]
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
      item.textContent = rule.title;
      list.append(item);
    }
    node.querySelector(".install").textContent = `harness-lint install ${pack.id}`;
    catalog.append(node);
  }
}

search.addEventListener("input", () => render(search.value));

loadCatalog()
  .then(() => render())
  .catch((error) => {
    catalog.textContent = `Failed to load catalog: ${error.message}`;
  });
