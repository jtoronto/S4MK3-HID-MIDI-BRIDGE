import { test, expect } from "bun:test";
import { startNativeSession } from "./native-session.js";

const root = process.env.WRITER_TRACE_TEST_ROOT;
if (root === undefined) throw new Error("WRITER_TRACE_TEST_ROOT is required");
const paths = {
  root,
  template: root + "/native/writer-fixture",
  injector: root + "/frida-inject-17.27.4-macos-arm64",
  agent: process.env.WRITER_TRACE_AGENT ?? root + "/writer-agent.js",
};

test("captures native buffers, caller identity and results without changing the calls", async () => {
  const session = await startNativeSession(paths);
  try {
    const ready = await session.wait((row) => row.kind === "ready");
    const finalReturn = session.wait(
      (row) =>
        row.kind === "batch" &&
        row.events.some((event) => event.kind === "return" && event.seq === 8),
    );
    expect(await session.batch(1)).toEqual({ batch: 1, failures: 0 });
    await finalReturn;
    const events = session.rows
      .filter((row) => row.kind === "batch")
      .flatMap((row) => row.events);
    const entries = events.filter((row) => row.kind === "entry");
    expect(entries.length).toBe(8);
    expect(
      entries.map((row) => ({
        api: row.api,
        handle: row.handle,
        role: row.reports[0].buffer_role,
        offset: row.reports[0].offset,
      })),
    ).toEqual([
      { api: "NtWriteFile", handle: "0x100", role: "write-input", offset: 0 },
      { api: "NtWriteFile", handle: "0x101", role: "write-input", offset: 0 },
      { api: "NtDeviceIoControlFile", handle: "0x200", role: "ioctl-input", offset: 3 },
      { api: "NtDeviceIoControlFile", handle: "0x201", role: "ioctl-output-at-entry", offset: 0 },
      { api: "HidD_SetOutputReport", handle: "0x300", role: "hid-output", offset: 0 },
      { api: "NtDeviceIoControlFile", handle: "0x300", role: "ioctl-input", offset: 0 },
      { api: "NtWriteFile", handle: "0x103", role: "write-input", offset: 0 },
      { api: "NtWriteFile", handle: "0x104", role: "write-input", offset: 0 },
    ]);
    expect(entries[0].reports[0].hex).toBe("3101e0fe000001e0fe0000");
    expect(entries[1].reports[0].hex).toBe("3101200100000120018629");
    expect(entries[1].reports[0].buffer_length).toBe(95);
    expect(entries.every((row) => row.tid > 0)).toBe(true);
    expect(entries.every((row) => Math.abs(row.utc_us - ready.utc_us) < 10e6)).toBe(true);
    const stack = events.find(
      (row) => row.kind === "stack" && row.id === entries[0].stack_id,
    );
    expect(stack.caller.module).toBe("fixture");
    expect(stack.frames.some((frame) => frame.module === "fixture")).toBe(true);
    expect(events.find((row) => row.kind === "return" && row.seq === 7).status).toBe(-1073741816);
    expect(events.find((row) => row.kind === "return" && row.seq === 8)).toMatchObject({
      status: 259, pending: true, accepted: false,
    });
    expect(session.rows.every((row) => row.pid === ready.pid)).toBe(true);
    session.command("STOP");
    expect(await session.wait((row) => row.kind === "stopped")).toMatchObject({
      reason: "operator", hook_count: 0, errors: 0, dropped: 0, outstanding_calls: 0,
    });
    const batchCount = session.rows.filter((row) => row.kind === "batch").length;
    expect(await session.batch(2)).toEqual({ batch: 2, failures: 0 });
    session.command("MARK ignored-after-stop");
    expect(session.rows.filter((row) => row.kind === "batch").length).toBe(batchCount);
  } finally {
    await session.close();
  }
});

test("deadline removes hooks and leaves the native target usable", async () => {
  const session = await startNativeSession({ ...paths, seconds: 1 });
  try {
    await session.wait((row) => row.kind === "ready");
    expect(await session.wait((row) => row.kind === "stopped")).toMatchObject({
      reason: "deadline", hook_count: 0,
    });
    expect(await session.batch(1)).toEqual({ batch: 1, failures: 0 });
    expect(session.rows.filter((row) => row.kind === "batch").length).toBe(0);
  } finally {
    await session.close();
  }
});
