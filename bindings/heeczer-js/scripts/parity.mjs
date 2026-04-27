import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { HeeczerClient } from "../dist/index.js";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "../../..");
const fixtureDir = path.resolve(
  process.env.HEECZER_PARITY_FIXTURE_DIR ??
    path.join(repoRoot, "core/schema/fixtures/events/valid"),
);
const referenceDir = process.env.HEECZER_PARITY_REFERENCE_DIR;
const baseUrl = process.env.HEECZER_PARITY_BASE_URL;

if (!referenceDir || !baseUrl) {
  throw new Error(
    "HEECZER_PARITY_REFERENCE_DIR and HEECZER_PARITY_BASE_URL are required",
  );
}

const client = new HeeczerClient({
  baseUrl,
  retry: { attempts: 3, backoffMs: 50 },
});

const fixtureNames = (await readdir(fixtureDir))
  .filter((name) => name.endsWith(".json"))
  .sort((left, right) => left.localeCompare(right));

if (fixtureNames.length === 0) {
  throw new Error(`no valid fixtures found in ${fixtureDir}`);
}

const failures = [];
for (const fixtureName of fixtureNames) {
  const eventPath = path.join(fixtureDir, fixtureName);
  const referencePath = path.join(
    referenceDir,
    `${path.basename(fixtureName, ".json")}.json`,
  );
  const event = JSON.parse(await readFile(eventPath, "utf8"));
  const expected = (await readFile(referencePath, "utf8")).trimEnd();
  const response = await client.testScorePipeline({ event });
  const actual = JSON.stringify(response.score);
  if (actual !== expected) {
    failures.push(`${fixtureName}: score JSON differed from Rust reference`);
  }
}

if (failures.length > 0) {
  throw new Error(failures.join("\n"));
}

console.log(`JS SDK parity passed for ${fixtureNames.length} fixture(s)`);
