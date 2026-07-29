#!/usr/bin/env node
// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

import { readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

export const SPDX_MARKER = "SPDX-License-Identifier: BUSL-1.1";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

const EXCLUDED_DIR_NAMES = new Set(["node_modules", "dist", "target"]);

const SCAN_ROOTS = [
  { root: join(repoRoot, "src"), extensions: [".rs"] },
  { root: join(repoRoot, "tests"), extensions: [".rs"] },
];

function shouldSkipDir(dirName) {
  return EXCLUDED_DIR_NAMES.has(dirName);
}

function walkDir(dir, extensions, files) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (!shouldSkipDir(entry.name)) {
        walkDir(full, extensions, files);
      }
      continue;
    }
    if (!entry.isFile()) {
      continue;
    }
    const ext = entry.name.includes(".")
      ? entry.name.slice(entry.name.lastIndexOf("."))
      : "";
    if (extensions.includes(ext)) {
      files.push(full);
    }
  }
}

export function collectLicenseHeaderFiles() {
  const files = [];
  for (const { root, extensions } of SCAN_ROOTS) {
    try {
      statSync(root);
    } catch {
      continue;
    }
    walkDir(root, extensions, files);
  }
  return files.sort();
}

export function readHeaderLines() {
  const path = join(repoRoot, "scripts", "license-header.txt");
  return readFileSync(path, "utf8")
    .split("\n")
    .map((line) => line.trimEnd())
    .filter((line) => line.length > 0);
}

export function hasLicenseHeader(content) {
  return content.includes(SPDX_MARKER);
}

export function formatHeaderLines(lines, prefix) {
  return lines.map((line) => `${prefix} ${line}`).join("\n") + "\n";
}

export function insertHeader(content) {
  const lines = readHeaderLines();
  const header = formatHeaderLines(lines, "//");

  if (hasLicenseHeader(content)) {
    return content;
  }

  const fileLines = content.split("\n");
  let insertAt = 0;

  if (fileLines[0]?.startsWith("#!") && !fileLines[0].startsWith("#![")) {
    insertAt = 1;
    if (fileLines[1] === "") {
      insertAt = 2;
    }
  }

  const before = fileLines.slice(0, insertAt).join("\n");
  const after = fileLines.slice(insertAt).join("\n");
  if (before.length === 0) {
    return after.length > 0 ? `${header}\n${after}` : header;
  }
  return after.length > 0
    ? `${before}\n${header}\n${after}`
    : `${before}\n${header}`;
}

export { repoRoot };
