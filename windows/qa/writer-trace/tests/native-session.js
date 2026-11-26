import * as fs from "node:fs/promises";
import { appendFileSync, watch } from "node:fs";

export async function startNativeSession(paths) {
  const directory = await fs.mkdtemp(paths.root + "/case-");
  const fixture = directory + "/fixture";
  await fs.copyFile(paths.template, fixture);
  await fs.chmod(fixture, 0o755);
  await Bun.$`mkfifo ${fixture + ".pipe"}`.quiet();
  const target = Bun.spawn([fixture], {
    env: process.env, stdin: "ignore", stdout: "ignore", stderr: "pipe", timeout: 25000,
  });
  const rows = [];
  const listeners = new Set();
  const decoder = new TextDecoder();
  let pending = "";
  let stderr = "";
  let fixturePipe;
  // POSIX frida-inject calls tcgetattr even in interactive mode. Windows
  // uses cooked pipe input directly; this PTY is only the native test transport.
  const child = Bun.spawn([
    paths.injector, "-p", String(target.pid), "-s", paths.agent,
    "-R", "qjs", "-i", "-P", JSON.stringify({ duration_seconds: paths.seconds ?? 180 }),
  ], {
    env: process.env, timeout: 20000,
    terminal: {
      cols: 160, rows: 30,
      data(_terminal, bytes) {
        appendFileSync(directory + "/agent.terminal", bytes);
        const lines = (pending + decoder.decode(bytes, { stream: true })).split("\n");
        pending = lines.pop();
        for (const line of lines) {
          if (!line.trim().startsWith("{")) { stderr += line; continue; }
          const row = JSON.parse(line);
          rows.push(row);
          for (const listener of [...listeners]) listener(row);
        }
      },
    },
  });
  function wait(predicate) {
    const existing = rows.find(predicate);
    if (existing !== undefined) return Promise.resolve(existing);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        listeners.delete(onRow);
        reject(new Error("Native agent event timeout: " + stderr));
      }, 10000);
      function onRow(row) {
        if (!predicate(row)) return;
        clearTimeout(timer); listeners.delete(onRow); resolve(row);
      }
      listeners.add(onRow);
    });
  }
  return {
    rows, wait,
    async batch(number) {
      const reply = new Promise((resolve, reject) => {
        const observer = watch(directory, async (_event, filename) => {
          if (filename !== "fixture.reply") return;
          try {
            const text = await fs.readFile(fixture + ".reply", "utf8");
            const found = text.trim().split("\n").filter(Boolean)
              .map((line) => JSON.parse(line)).find((row) => row.batch === number);
            if (found !== undefined) {
              clearTimeout(timer); observer.close(); resolve(found);
            }
          } catch (error) {
            observer.close(); clearTimeout(timer); reject(error);
          }
        });
        const timer = setTimeout(() => {
          observer.close(); reject(new Error("Native fixture reply timeout"));
        }, 10000);
      });
      if (fixturePipe === undefined) fixturePipe = await fs.open(fixture + ".pipe", "w");
      await fixturePipe.write(`BATCH ${number}\n`);
      return await reply;
    },
    command(text) { child.terminal.write(text + "\n"); },
    async close() {
      child.terminal.write("\x04");
      await fixturePipe?.close();
      const deadline = setTimeout(() => child.kill("SIGTERM"), 2000);
      const code = await child.exited;
      clearTimeout(deadline);
      child.terminal.close();
      target.kill("SIGTERM");
      await target.exited;
      await fs.writeFile(directory + "/observed.json", JSON.stringify({ code, stderr, rows }, null, 2));
      return { code, stderr, directory };
    },
  };
}
