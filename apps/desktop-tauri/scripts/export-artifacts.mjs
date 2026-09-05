import { createHash } from "node:crypto";
import { cp, mkdir, readFile, readdir, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const desktopDirectory = path.resolve(scriptDirectory, "..");
const projectDirectory = path.resolve(desktopDirectory, "../..");
const outputDirectory = path.join(projectDirectory, "artifacts");
const releaseDirectory = path.join(desktopDirectory, "src-tauri", "target", "release");
const packageJson = JSON.parse(await readFile(path.join(desktopDirectory, "package.json"), "utf8"));

await mkdir(outputDirectory, { recursive: true });

const copiedFiles = [];

if (process.platform === "darwin") {
  const appSource = path.join(releaseDirectory, "bundle", "macos", "CodexMeter.app");
  const architecture = process.arch === "arm64" ? "arm64" : "x64";
  const appDestination = path.join(outputDirectory, "CodexMeter.app");
  await requirePath(appSource, "macOS application");
  await rm(appDestination, { recursive: true, force: true });
  await cp(appSource, appDestination, { recursive: true, preserveTimestamps: true });

  const dmgDirectory = path.join(releaseDirectory, "bundle", "dmg");
  const dmgSource = await firstFile(dmgDirectory, (name) => name.endsWith(".dmg"));
  const dmgDestination = path.join(
    outputDirectory,
    `CodexMeter-macOS-${packageJson.version}-${architecture}.dmg`,
  );
  await cp(dmgSource, dmgDestination);
  copiedFiles.push(dmgDestination);
} else if (process.platform === "win32") {
  const executableSource = path.join(releaseDirectory, "codexmeter.exe");
  await requirePath(executableSource, "Windows portable executable");
  const executableDestination = path.join(outputDirectory, "CodexMeter.exe");
  await cp(executableSource, executableDestination);
  copiedFiles.push(executableDestination);

  const installerDirectory = path.join(releaseDirectory, "bundle", "nsis");
  const installerSource = await firstFile(installerDirectory, (name) => name.endsWith(".exe"));
  const installerDestination = path.join(
    outputDirectory,
    `CodexMeter-Windows-Setup-${packageJson.version}-x64.exe`,
  );
  await cp(installerSource, installerDestination);
  copiedFiles.push(installerDestination);
} else {
  throw new Error(`Artifact export is not configured for ${process.platform}.`);
}

const checksums = [];
for (const file of copiedFiles) {
  const bytes = await readFile(file);
  checksums.push(`${createHash("sha256").update(bytes).digest("hex")}  ${path.basename(file)}`);
}
await writeFile(path.join(outputDirectory, "SHA256SUMS.txt"), `${checksums.join("\n")}\n`);

for (const file of copiedFiles) console.log(path.relative(projectDirectory, file));
console.log(path.relative(projectDirectory, path.join(outputDirectory, "SHA256SUMS.txt")));

async function requirePath(target, label) {
  try {
    await stat(target);
  } catch {
    throw new Error(`Missing ${label}: ${target}. Run the platform package command first.`);
  }
}

async function firstFile(directory, matches) {
  let names;
  try {
    names = await readdir(directory);
  } catch {
    throw new Error(`Missing bundle directory: ${directory}. Run the platform package command first.`);
  }
  const name = names.sort().find(matches);
  if (!name) throw new Error(`No matching bundle found in ${directory}.`);
  return path.join(directory, name);
}
