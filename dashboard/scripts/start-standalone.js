const fs = require("fs");
const path = require("path");

const rootDir = path.resolve(__dirname, "..");
const sourceStaticDir = path.join(rootDir, ".next", "static");
const targetStaticDir = path.join(
  rootDir,
  ".next",
  "standalone",
  ".next",
  "static"
);
const sourcePublicDir = path.join(rootDir, "public");
const targetPublicDir = path.join(rootDir, ".next", "standalone", "public");
const standaloneServer = path.join(rootDir, ".next", "standalone", "server.js");

if (!fs.existsSync(sourceStaticDir) || !fs.existsSync(standaloneServer)) {
  throw new Error("Standalone build output is missing. Run `next build` first.");
}

copyDirectoryContents(sourceStaticDir, targetStaticDir);

if (fs.existsSync(sourcePublicDir)) {
  copyDirectoryContents(sourcePublicDir, targetPublicDir);
}

require(standaloneServer);

function copyDirectoryContents(sourceDir, targetDir) {
  fs.mkdirSync(targetDir, { recursive: true });

  for (const entry of fs.readdirSync(sourceDir)) {
    fs.cpSync(path.join(sourceDir, entry), path.join(targetDir, entry), {
      force: true,
      recursive: true,
    });
  }
}
