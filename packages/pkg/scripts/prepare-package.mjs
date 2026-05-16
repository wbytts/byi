import { chmodSync, copyFileSync, existsSync, mkdirSync, rmSync, statSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const packageDir = path.resolve(import.meta.dirname, "..");
const repoRoot = path.resolve(packageDir, "..", "..");
const sourceDir = path.resolve(process.env.BYI_PKG_SOURCE_DIR ?? path.join(repoRoot, "target"));
const distDir = path.join(packageDir, "dist");
const KNOWN_TARGETS = [
  { triple: "x86_64-apple-darwin", binary: "byi" },
  { triple: "aarch64-apple-darwin", binary: "byi" },
  { triple: "x86_64-unknown-linux-gnu", binary: "byi" },
  { triple: "aarch64-unknown-linux-gnu", binary: "byi" },
  { triple: "x86_64-pc-windows-msvc", binary: "byi.exe" },
  { triple: "aarch64-pc-windows-msvc", binary: "byi.exe" }
];

function getHostTriple() {
  const output = execFileSync("rustc", ["-vV"], {
    cwd: repoRoot,
    encoding: "utf8"
  });
  const hostLine = output
    .split("\n")
    .find((line) => line.startsWith("host: "));

  if (!hostLine) {
    throw new Error("无法从 rustc -vV 解析 host triple。");
  }

  return hostLine.slice("host: ".length).trim();
}

function copyIfExists(sourcePath, targetInfo) {
  if (!existsSync(sourcePath) || !statSync(sourcePath).isFile()) {
    return false;
  }

  const outputDir = path.join(distDir, targetInfo.triple);
  const outputPath = path.join(outputDir, targetInfo.binary);
  mkdirSync(outputDir, { recursive: true });
  copyFileSync(sourcePath, outputPath);

  if (targetInfo.binary === "byi") {
    chmodSync(outputPath, 0o755);
  }

  return true;
}

function collectBinaries() {
  const copied = [];
  const hostBinary = process.platform === "win32" ? "byi.exe" : "byi";
  const rootReleasePath = path.join(sourceDir, "release", hostBinary);

  if (existsSync(rootReleasePath)) {
    const hostTriple = getHostTriple();
    if (KNOWN_TARGETS.some((item) => item.triple === hostTriple && item.binary === hostBinary)) {
      if (copyIfExists(rootReleasePath, { triple: hostTriple, binary: hostBinary })) {
        copied.push(hostTriple);
      }
    }
  }

  for (const targetInfo of KNOWN_TARGETS) {
    const sourcePath = path.join(sourceDir, targetInfo.triple, "release", targetInfo.binary);
    if (copyIfExists(sourcePath, targetInfo) && !copied.includes(targetInfo.triple)) {
      copied.push(targetInfo.triple);
    }
  }

  return copied;
}

rmSync(distDir, { recursive: true, force: true });
mkdirSync(distDir, { recursive: true });

const copiedTargets = collectBinaries();

if (copiedTargets.length === 0) {
  throw new Error(`未找到可发布的 Rust 构建产物。当前扫描目录: ${sourceDir}`);
}

console.log(`已复制 ${copiedTargets.length} 个 target: ${copiedTargets.join(", ")}`);
