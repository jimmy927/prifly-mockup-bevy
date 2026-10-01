/**
 * Validates every recipe in mockup-stacks/ against prifly's loader schema (Bun only, no dependencies):
 * frontmatter parses as YAML, required keys have the right shape, the id is the file name, the fixture exists,
 * the body has Steps / Gotchas / States, and nothing private leaked. Exit 1 on any problem.
 */
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { isAbsolute, join, normalize } from "node:path";

const root = join(import.meta.dir, "..", "mockup-stacks");
const problems: string[] = [];
const FRONT = /^﻿?---[ \t]*\r?\n([\s\S]*?)\r?\n---[ \t]*(?:\r?\n|$)/;
const scalar = (v: unknown) => typeof v === "string" || typeof v === "number";
const names = (v: unknown) => v === undefined || (Array.isArray(v) && v.every(scalar));

function fail(file: string, message: string): void {
  problems.push(`${file}: ${message}`);
}

function checkFrontmatter(file: string, raw: Record<string, unknown>): void {
  if (typeof raw.id !== "string" || !/^[a-z0-9][a-z0-9-]*$/.test(raw.id))
    fail(file, "id: lower-case words joined by hyphens");
  else if (`${raw.id}.md` !== file) fail(file, `id "${raw.id}" does not match the file name`);
  for (const key of ["name", "route"]) {
    if (!scalar(raw[key]) || String(raw[key]) === "") fail(file, `${key}: required text`);
  }
  if (!scalar(raw.verified) || !/^\d{4}-\d{2}-\d{2}$/.test(String(raw.verified)))
    fail(file, "verified: a date, YYYY-MM-DD");
  for (const key of ["aliases", "renderedBy"]) {
    if (!names(raw[key])) fail(file, `${key}: a list of text`);
  }
  const detect = raw.detect;
  if (typeof detect !== "object" || detect === null || Array.isArray(detect))
    fail(file, "detect: required mapping (may be {})");
  else
    for (const key of ["deps", "files", "globs"]) {
      if (!names((detect as Record<string, unknown>)[key])) fail(file, `detect.${key}: a list of text`);
    }
  if (raw.priority !== undefined && !Number.isInteger(raw.priority)) fail(file, "priority: an integer");
  for (const key of ["versions", "kind", "where", "toolchain", "verifiedBy", "fixture"]) {
    if (raw[key] !== undefined && !scalar(raw[key])) fail(file, `${key}: text`);
  }
  if (typeof raw.fixture === "string") {
    if (isAbsolute(raw.fixture) || normalize(raw.fixture).startsWith(".."))
      fail(file, "fixture: a path inside the recipe's folder");
    else if (!existsSync(join(root, raw.fixture, "build.sh"))) fail(file, `fixture ${raw.fixture} has no build.sh`);
  }
}

const recipes = readdirSync(root).filter((f) => f.endsWith(".md") && statSync(join(root, f)).isFile());
if (recipes.length === 0) problems.push("no recipes in mockup-stacks/");
for (const file of recipes) {
  const text = readFileSync(join(root, file), "utf8");
  const match = FRONT.exec(text);
  if (match === null) {
    fail(file, "no frontmatter block (--- ... --- at the top)");
    continue;
  }
  let raw: unknown;
  try {
    raw = Bun.YAML.parse(match[1] ?? "");
  } catch (caught) {
    fail(file, `frontmatter is not valid YAML: ${caught instanceof Error ? caught.message : caught}`);
    continue;
  }
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) {
    fail(file, "frontmatter is not a mapping");
    continue;
  }
  checkFrontmatter(file, raw as Record<string, unknown>);
  for (const section of ["## Steps", "## Gotchas", "## States"]) {
    if (!text.includes(`\n${section}\n`)) fail(file, `missing ${section}`);
  }
  if (/\/home\/[a-z]|\/mnt\/c\/Users\/|C:\\Users\\/i.test(text)) fail(file, "contains a home-directory path");
}

if (problems.length > 0) {
  for (const problem of problems) console.log(`FAIL  ${problem}`);
  process.exit(1);
}
console.log(`${recipes.length} recipe(s) valid, 0 problems`);
