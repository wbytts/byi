#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const TARGET_MAP = {
  darwin: {
    x64: { triple: "x86_64-apple-darwin", binary: "byi" },
    arm64: { triple: "aarch64-apple-darwin", binary: "byi" }
  },
  linux: {
    x64: { triple: "x86_64-unknown-linux-gnu", binary: "byi" },
    arm64: { triple: "aarch64-unknown-linux-gnu", binary: "byi" }
  },
  win32: {
    x64: { triple: "x86_64-pc-windows-msvc", binary: "byi.exe" },
    arm64: { triple: "aarch64-pc-windows-msvc", binary: "byi.exe" }
  }
};

function resolveBinary() {
  const platformTargets = TARGET_MAP[process.platform];
  if (!platformTargets) {
    throw new Error(`不支持的运行平台: ${process.platform}`);
  }

  const target = platformTargets[process.arch];
  if (!target) {
    throw new Error(`不支持的系统架构: ${process.platform}/${process.arch}`);
  }

  const binaryPath = path.resolve(
    __dirname,
    "..",
    "dist",
    target.triple,
    target.binary
  );

  if (!fs.existsSync(binaryPath)) {
    throw new Error(
      `npm 包内缺少当前平台的 Rust 产物: ${target.triple}/${target.binary}`
    );
  }

  return binaryPath;
}

const binaryPath = resolveBinary();

if (process.platform !== "win32") {
  try {
    fs.chmodSync(binaryPath, 0o755);
  } catch (error) {
    console.error(`无法为二进制设置执行权限: ${error.message}`);
    process.exit(1);
  }
}

const result = spawnSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
