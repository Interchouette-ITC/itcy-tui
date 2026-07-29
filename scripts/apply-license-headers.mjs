#!/usr/bin/env node
// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

import { readFileSync, writeFileSync } from "node:fs";
import { relative } from "node:path";
import {
  collectLicenseHeaderFiles,
  hasLicenseHeader,
  insertHeader,
  repoRoot,
} from "./license-header-lib.mjs";

const dryRun = process.argv.includes("--dry-run");

const files = collectLicenseHeaderFiles();
let changed = 0;

for (const file of files) {
  const content = readFileSync(file, "utf8");
  if (hasLicenseHeader(content)) {
    continue;
  }
  const updated = insertHeader(content);
  if (updated === content) {
    continue;
  }
  changed += 1;
  if (dryRun) {
    console.log(`would update: ${relative(repoRoot, file)}`);
  } else {
    writeFileSync(file, updated);
  }
}

if (dryRun) {
  console.log(
    `dry-run: ${changed} file(s) would be updated (${files.length} scanned)`,
  );
} else {
  console.log(`updated ${changed} file(s) (${files.length} scanned)`);
}
