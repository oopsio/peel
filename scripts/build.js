import { spawn } from "child_process";
import { copyFile, mkdir, rm } from "node:fs/promises";
import { join } from "node:path";

const DIST_DIR = "dist";

async function runCommand(command, args, cwd) {
  return new Promise((resolve, reject) => {
    console.log(`\x1b[36mRunning:\x1b[0m ${command} ${args.join(" ")} in ${cwd}`);
    const proc = spawn(command, args, { cwd, stdio: "inherit", shell: true });
    proc.on("close", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${command} failed with code ${code}`));
    });
  });
}

async function build() {
  try {
    // 1. Prepare dist directory
    await rm(DIST_DIR, { recursive: true, force: true });
    await mkdir(DIST_DIR);

    // 2. Build Peel (Runtime)
    console.log("\n\x1b[32m--- Building Peel Runtime ---\x1b[0m");
    await runCommand("cargo", ["build", "--release"], ".");

    // 3. Build Pepm (Package Manager)
    console.log("\n\x1b[32m--- Building Pepm Package Manager ---\x1b[0m");
    await runCommand("cargo", ["build", "--release"], "crates/pepm");

    // 4. Copy Binaries
    const peelExe = process.platform === "win32" ? "peel.exe" : "peel";
    const pepmExe = process.platform === "win32" ? "pepm.exe" : "pepm";

    await copyFile(join("target/release", peelExe), join(DIST_DIR, peelExe));
    await copyFile(join("crates/pepm/target/release", pepmExe), join(DIST_DIR, pepmExe));

    console.log(`\n\x1b[32mBuild completed! Binaries are in ./${DIST_DIR}\x1b[0m`);
  } catch (err) {
    console.error(`\x1b[31mBuild failed:\x1b[0m ${err.message}`);
    process.exit(1);
  }
}

build();
