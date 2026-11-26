import * as fs from "node:fs/promises";
import * as path from "node:path";

const [injector, destination] = process.argv.slice(2);
if (injector === undefined || destination === undefined)
  throw new Error("Usage: bun build-package.js VERIFIED_WINDOWS_INJECTOR NEW_DIRECTORY");
const source = import.meta.dir;
const injectorSha = "3415601f709ffa1aefa8e5aa8825ec91ea36f94ee5c61e1e8f1441a13f384edc";
function sha(bytes) {
  return new Bun.CryptoHasher("sha256").update(bytes).digest("hex");
}
if (sha(await Bun.file(injector).arrayBuffer()) !== injectorSha)
  throw new Error("Injector differs from the verified Frida 17.27.4 Windows x64 asset");
await fs.mkdir(destination, { recursive: false });
const runtimeFiles = [
  "WriterSession.cs",
  "WriterSession.Process.cs",
  "WriterSession.Protocol.cs",
  "TraceDeadline.cs",
  "Start-WriterTrace.ps1",
  "Etw-Trace.ps1",
  "Usb-WriterTrace.wprp",
  "08-writer-trace.cmd",
  "README.md",
];
for (const file of runtimeFiles)
  await fs.copyFile(path.join(source, file), path.join(destination, file));
await fs.copyFile(injector, path.join(destination, "frida-inject.exe"));
const build = await Bun.build({
  entrypoints: [path.join(source, "agent.js")],
  target: "browser",
  format: "iife",
  minify: false,
});
if (!build.success) throw new AggregateError(build.logs, "Agent compilation failed");
await Bun.write(path.join(destination, "writer-agent.js"), build.outputs[0]);
const licenseUrl = "https://raw.githubusercontent.com/frida/frida-core/17.27.4/COPYING";
const license = await fetch(licenseUrl);
if (!license.ok) throw new Error("Official Frida license download failed");
await Bun.write(path.join(destination, "FRIDA-LICENSE.txt"), license);
const files = [];
for (const file of [...runtimeFiles, "frida-inject.exe", "writer-agent.js", "FRIDA-LICENSE.txt"])
  files.push({ path: file, sha256: sha(await Bun.file(path.join(destination, file)).arrayBuffer()) });
const sourceFiles = [];
for (const file of ["agent.js", "hooks.js", "reports.js", "build-package.js"])
  sourceFiles.push({ path: file, sha256: sha(await Bun.file(path.join(source, file)).arrayBuffer()) });
const identity = {
  version: 1,
  preparedUtc: new Date().toISOString(),
  mode: "stopped; no live NI attachment or controller test",
  fridaVersion: "17.27.4",
  fridaRelease: "https://github.com/frida/frida/releases/tag/17.27.4",
  fridaCompressedAssetSha256: "68474826ac22767b8c5711fde83cb2f468deddcfdacaffd1b2fce547419e0a01",
  baseHostSha256: "538ab82f98042f46ff461c931b72792a21e9c965a16c2c5777b0457babc466d6",
  baseEngineSha256: "75bf26f55c1cfbe8bde7e25bf7e8e325922f6d044e8905235b4a5bdf8747618c",
  files,
  sourceFiles,
};
await Bun.write(path.join(destination, "TRACE-IDENTITY.json"), JSON.stringify(identity, null, 2) + "\n");
console.log(JSON.stringify({ package: destination, files: files.length, stopped: true }));
