#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const { resolve } = require("node:path");

const localBinary = resolve(__dirname, "..", "vendor", process.platform, process.arch, "harness-lint");
const binary = existsSync(localBinary) ? localBinary : "harness-lint";
const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
