import { renameSync } from "node:fs";
import { join } from "node:path";

const outDir = new URL("../dist/cjs/", import.meta.url);
renameSync(
  join(outDir.pathname, "index.js"),
  join(outDir.pathname, "index.cjs"),
);
