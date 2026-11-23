import { attachWriters } from "./hooks.js";

// Mutation is confined to this bounded, short-lived collector.
const queue = [];
const callCounts = {};
const stacks = new Map();
let sequence = 0;
let dropped = 0;
let errors = 0;
let active = 0;
let running = false;
let attachment;
let interval;
let deadline;
let clock;

function output(event) {
  // The injector explicitly flushes this channel. console.log can remain
  // buffered when stdout is redirected, deadlocking the ready/stop handshake.
  send(["frida:stdout", JSON.stringify({ ...event, pid: Process.id }) + "\n"]);
}
function enqueue(event) {
  if (queue.length < 8192) queue.push(event);
  else dropped++;
}
function flush() {
  if (queue.length !== 0)
    output({ kind: "batch", events: queue.splice(0), last_seq: sequence });
}
function frame(address) {
  const module = Process.findModuleByAddress(address);
  return {
    address: address.toString(),
    module: module === null ? null : module.name,
    path: module === null ? null : module.path,
    offset: module === null ? null : address.sub(module.base).toString(),
  };
}
function enter(call) {
  const utc = clock();
  const layout = call.matches.map((report) =>
    `${report.buffer_role}:${report.report_id}:${report.offset}:${report.buffer_length}`,
  ).join("/");
  const key = `${call.api}/${call.tid}/${call.handle}/${call.returnAddress}/${call.ioctl}/${layout}`;
  let stackId = stacks.get(key);
  if (stackId === undefined) {
    stackId = stacks.size + 1;
    if (stackId > 256) {
      dropped++;
      stackId = null;
    } else {
      stacks.set(key, stackId);
      let frames = [];
      let unwindError = null;
      try {
        frames = Thread.backtrace(call.context, Backtracer.ACCURATE).map(frame);
      } catch (error) {
        if (!(error instanceof Error)) throw error;
        unwindError = error.message;
      }
      enqueue({
        kind: "stack",
        id: stackId,
        api: call.api,
        tid: call.tid,
        caller: frame(call.returnAddress),
        frames,
        unwind_error: unwindError,
      });
    }
  }
  const id = ++sequence;
  active++;
  enqueue({
    kind: "entry",
    seq: id,
    utc_us: utc,
    api: call.api,
    tid: call.tid,
    handle: call.handle,
    ioctl: call.ioctl,
    reports: call.matches,
    stack_id: stackId,
    stack_is_stream_sample: true,
    caller_address: call.returnAddress.toString(),
  });
  return id;
}
function stop(reason) {
  if (!running) return;
  running = false;
  attachment.detach();
  clearInterval(interval);
  clearTimeout(deadline);
  flush();
  output({
    kind: "summary",
    last_seq: sequence,
    calls: callCounts,
    dropped,
    errors,
    outstanding_calls: active,
  });
  output({
    kind: "stopped",
    reason,
    hook_count: 0,
    dropped,
    errors,
    outstanding_calls: active,
  });
}
rpc.exports = {
  init(_stage, parameters) {
    const seconds = parameters.duration_seconds ?? 180;
    if (!Number.isInteger(seconds) || seconds < 1 || seconds > 180)
      throw new Error("duration_seconds must be an integer from 1 through 180");
    const kernel = Process.getModuleByName("kernel32.dll");
    const now = new NativeFunction(
      kernel.getExportByName("GetSystemTimePreciseAsFileTime"),
      "void",
      ["pointer"],
      { scheduling: "exclusive" },
    );
    const timestamp = Memory.alloc(8);
    clock = () => {
      now(timestamp);
      return Number(
        (BigInt(timestamp.readU64().toString()) - 116444736000000000n) / 10n,
      );
    };
    attachment = attachWriters({
      enter,
      leave(id, result) {
        active--;
        enqueue({ kind: "return", seq: id, utc_us: clock(), ...result });
      },
      called(api) {
        callCounts[api] = (callCounts[api] ?? 0) + 1;
      },
      error(api, error) {
        errors++;
        enqueue({ kind: "error", api, message: error.message, utc_us: clock() });
      },
      hooks(hooks) {
        output({ kind: "hooks", hooks });
      },
    });
    running = true;
    interval = setInterval(flush, 100);
    deadline = setTimeout(() => stop("deadline"), seconds * 1000);
    output({
      kind: "ready",
      version: 1,
      arch: Process.arch,
      platform: Process.platform,
      utc_us: clock(),
      hooks: attachment.hooks,
      modules: Process.enumerateModules().map((module) => ({
        name: module.name,
        path: module.path,
        base: module.base.toString(),
        size: module.size,
      })),
    });
  },
  onFridaStdin(text) {
    const command = text.trim();
    if (command === "STOP") stop("operator");
    else if (command.startsWith("MARK ") && running) {
      flush();
      output({ kind: "mark", text: command.slice(5), utc_us: clock() });
    } else throw new Error("Expected STOP or MARK text");
  },
};
