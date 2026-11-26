const writeFile = { name: "NtWriteFile", buffer: 5, length: 6 };
const transceive = { name: "NtFsControlFile", buffer: 6, length: 7 };
const transceiveCode = 0x0011c017;
const display = 0x03647344;
const setUIntProperty = 0x03567375;
const jogTensionKey = 0x4a4f4754;
const lifecycle = new Set([
  0x03447500, 0x03444900, 0x03404300, 0x03434300, 0x03432d00,
  0x0349734e, 0x03497354, 0x03497374, 0x03445200, 0x03444100,
]);
const markers = new Set(["READY_IDLE", "TT", "PLAY", "PAUSE", "JOG", "QUIT",
  "TENSION_A", "TENSION_B", "RESTORE"]);
const maxRequests = 24000;
const maxJogBytes = 4096;
let sequence = 0;
let phase = "STARTUP";
let errors = 0;
let stopped = false;
const shownDisplays = new Set();
const observedPipes = new Map();
const listeners = [];
let deadline;

function log(kind, fields = {}) {
  send(["frida:stdout", JSON.stringify({
    kind, time_utc: new Date().toISOString(), pid: Process.id, phase, ...fields,
  }) + "\n"]);
}

function message(text) {
  send(["frida:stderr", text + "\n"]);
}

function stop(reason) {
  if (stopped) return;
  stopped = true;
  clearTimeout(deadline);
  for (const listener of listeners) listener.detach();
  Interceptor.flush();
  log("stopped", { reason, requests: sequence, errors });
  message("Capture hooks stopped (" + reason + ").");
}

function attach(nt, spec, fileType, serverPid, pidBuffer) {
  const address = nt.getExportByName(spec.name);
  const listener = Interceptor.attach(address, {
    onEnter(args) {
      this.traceSequence = 0;
      if (stopped || sequence >= maxRequests) return;
      if (spec.name === transceive.name && args[5].toUInt32() !== transceiveCode) return;
      const size = args[spec.length].toUInt32();
      const buffer = args[spec.buffer];
      if (size < 4 || buffer.isNull()) return;
      const lastError = this.lastError;
      try {
        const id = buffer.readU32();
        const jog = id >>> 16 === 0x036a;
        const lcd = id === display;
        const tension = id === setUIntProperty && size >= 8 && buffer.add(4).readU32() === jogTensionKey;
        if (!jog && !lcd && !tension && !lifecycle.has(id)) return;
        const handle = args[0];
        if (fileType(handle) !== 3) return;
        const handleText = handle.toString();
        if (lcd) {
          const screen = size >= 8 ? buffer.add(4).readU32() : null;
          const key = `${phase}/${handleText}/${screen}`;
          if (shownDisplays.has(key)) return;
          shownDisplays.add(key);
        }
        const peer = serverPid(handle, pidBuffer) !== 0 ? pidBuffer.readU32() : null;
        const retain = Math.min(size, jog ? maxJogBytes : lcd ? 20 : tension ? 12 : 8);
        const bytes = buffer.readByteArray(retain);
        if (bytes === null) throw new Error("Request bytes are unreadable");
        const hex = Array.from(new Uint8Array(bytes), byte => byte.toString(16).padStart(2, "0")).join("");
        const seq = ++sequence;
        observedPipes.set(handleText, peer);
        this.traceSequence = seq;
        log("request", {
          seq, api: spec.name, tid: this.threadId, handle: handleText, server_pid: peer,
          command: "0x" + id.toString(16).padStart(8, "0"), bytes: size, captured_bytes: retain,
          complete: retain === size, hex,
          ...(tension ? { property: "JOGT", value: size >= 12 ? buffer.add(8).readU32() : null } : {}),
        });
      } catch (error) {
        errors++;
        if (errors <= 8) log("read_error", { api: spec.name, error: String(error) });
      } finally {
        this.lastError = lastError;
      }
    },
    onLeave(value) {
      if (stopped || this.traceSequence === 0) return;
      const status = value.toUInt32();
      log("return", { seq: this.traceSequence, status: "0x" + status.toString(16).padStart(8, "0") });
      if (sequence >= maxRequests) stop("record_limit");
    },
  });
  listeners.push(listener);
}

rpc.exports = {
  init() {
    try {
      const target = Process.mainModule;
      if (Process.platform !== "windows" || Process.arch !== "x64" ||
          !/^Traktor(?: Pro(?: [0-9]+)?)?\.exe$/i.test(target.name)) {
        throw new Error("Expected the stock Windows x64 Traktor executable");
      }
      const nt = Process.getModuleByName("ntdll.dll");
      const kernel = Process.getModuleByName("kernel32.dll");
      const options = { scheduling: "exclusive", traps: "none" };
      const fileType = new NativeFunction(kernel.getExportByName("GetFileType"), "uint32", ["pointer"], options);
      const serverPid = new NativeFunction(kernel.getExportByName("GetNamedPipeServerProcessId"),
        "int", ["pointer", "pointer"], options);
      const pidBuffer = Memory.alloc(4);
      attach(nt, writeFile, fileType, serverPid, pidBuffer);
      attach(nt, transceive, fileType, serverPid, pidBuffer);
      listeners.push(Interceptor.attach(nt.getExportByName("NtClose"), {
        onEnter(args) {
          this.traceClose = null;
          if (stopped) return;
          const handle = args[0].toString();
          if (!observedPipes.has(handle)) return;
          this.traceClose = { handle, server_pid: observedPipes.get(handle), tid: this.threadId };
          observedPipes.delete(handle);
        },
        onLeave(value) {
          if (stopped || this.traceClose === null) return;
          log("pipe_close", { ...this.traceClose,
            status: "0x" + value.toUInt32().toString(16).padStart(8, "0") });
        },
      }));
      deadline = setTimeout(() => stop("deadline"), 180000);
      log("ready", { target: target.name, hooks: [writeFile.name, transceive.name, "NtClose"], seconds: 180 });
      message("CAPTURE READY. Mark READY_IDLE after Traktor is open and Deck A is paused.");
    } catch (error) {
      log("startup_error", { error: String(error) });
      stop("startup_error");
    }
  },
  onFridaStdin(value) {
    const marker = value.trim();
    if (marker === "STOP_CAPTURE") {
      stop("operator");
    } else if (!stopped && markers.has(marker)) {
      phase = marker;
      shownDisplays.clear();
      log("mark", { marker });
      message("MARKED " + marker);
    }
  },
};
