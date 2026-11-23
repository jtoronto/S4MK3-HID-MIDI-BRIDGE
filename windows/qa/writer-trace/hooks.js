import { reportsAt } from "./reports.js";

export function attachWriters(trace) {
  const listeners = [];
  const installed = new Set();
  const hooks = [];
  function attach(module, name, decode) {
    const address = module.findExportByName(name);
    if (address === null || installed.has(address.toString())) return;
    installed.add(address.toString());
    listeners.push(
      Interceptor.attach(address, {
        onEnter(args) {
          const savedError = this.lastError;
          this.writerCall = null;
          try {
            const decoded = decode(args);
            trace.called(name);
            const matches = decoded.buffers.flatMap((buffer) =>
              reportsAt(buffer.pointer, buffer.length, decoded.embedded).map(
                (report) => ({ ...report, buffer_role: buffer.role }),
              ),
            );
            if (matches.length === 0) return;
            this.writerCall = trace.enter({
              api: name,
              tid: this.threadId,
              handle: args[0].toString(),
              ioctl: decoded.ioctl,
              matches,
              context: this.context,
              returnAddress: this.returnAddress,
            });
          } catch (error) {
            if (!(error instanceof Error)) throw error;
            trace.error(name, error);
          } finally {
            if (savedError !== undefined) this.lastError = savedError;
          }
        },
        onLeave(value) {
          if (this.writerCall === null) return;
          const savedError = this.lastError;
          try {
            const status = value.toInt32();
            trace.leave(this.writerCall, {
              status,
              pending: name !== "HidD_SetOutputReport" && status === 0x103,
              accepted:
                name === "HidD_SetOutputReport"
                  ? status !== 0
                  : status >= 0 && status !== 0x103,
            });
          } finally {
            if (savedError !== undefined) this.lastError = savedError;
          }
        },
      }),
    );
    hooks.push({ module: module.name, api: name, address: address.toString() });
  }
  const nt = Process.getModuleByName("ntdll.dll");
  attach(nt, "NtWriteFile", (args) => ({
    embedded: false,
    ioctl: null,
    buffers: [
      { pointer: args[5], length: args[6].toUInt32(), role: "write-input" },
    ],
  }));
  attach(nt, "NtDeviceIoControlFile", (args) => ({
    embedded: true,
    ioctl: args[5].toUInt32(),
    // Output at entry is only a candidate: it may be an uninitialized receive
    // buffer. No matching IOCTL bytes alone prove a device-bound write.
    buffers: [
      { pointer: args[6], length: args[7].toUInt32(), role: "ioctl-input" },
      { pointer: args[8], length: args[9].toUInt32(), role: "ioctl-output-at-entry" },
    ],
  }));
  if (hooks.length !== 2) throw new Error("Required NT write hooks unavailable");
  const observer = Process.attachModuleObserver({
    onAdded(module) {
      if (module.name.toLowerCase() !== "hid.dll") return;
      attach(module, "HidD_SetOutputReport", (args) => ({
        embedded: false,
        ioctl: null,
        buffers: [
          { pointer: args[1], length: args[2].toUInt32(), role: "hid-output" },
        ],
      }));
      trace.hooks(hooks);
    },
  });
  return {
    hooks,
    detach() {
      observer.detach();
      for (const listener of listeners) listener.detach();
      Interceptor.flush();
    },
  };
}
