#!/usr/bin/env node
// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

import { readFileSync } from "node:fs";
import { relative } from "node:path";
import {
  collectLicenseHeaderFiles,
  hasLicenseHeader,
  repoRoot,
} from "./license-header-lib.mjs";

const files = collectLicenseHeaderFiles();
const missing = [];

for (const file of files) {
  const content = readFileSync(file, "utf8");
  if (!hasLicenseHeader(content)) {
    missing.push(relative(repoRoot, file));
  }
}

if (missing.length > 0) {
  console.error(`Missing license header in ${missing.length} file(s):`);
  for (const path of missing) {
    console.error(`  ${path}`);
  }
  process.exit(1);
}

console.log(`OK: ${files.length} file(s) have license headers`);
