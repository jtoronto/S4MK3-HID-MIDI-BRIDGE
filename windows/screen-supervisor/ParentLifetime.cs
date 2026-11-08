using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace S4ScreenSupervisor;

internal sealed class ParentLifetime : IDisposable
{
    private readonly EventWaitHandle parent;
    private readonly RegisteredWaitHandle registration;
    internal Task Exited { get; }

    private ParentLifetime(SafeWaitHandle handle)
    {
        parent = new EventWaitHandle(false, EventResetMode.ManualReset) { SafeWaitHandle = handle };
        var exit = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        Exited = exit.Task;
        registration = ThreadPool.RegisterWaitForSingleObject(parent, (_, _) => exit.TrySetResult(),
            null, Timeout.Infinite, executeOnlyOnce: true);
    }

    internal static ParentLifetime Open()
    {
        using var snapshot = CreateToolhelp32Snapshot(2, 0);
        if (snapshot.IsInvalid) throw new Win32Exception(Marshal.GetLastWin32Error());
        var entry = new ProcessEntry { Size = (uint)Marshal.SizeOf<ProcessEntry>(), File = "" };
        if (!Process32FirstW(snapshot, ref entry)) throw new Win32Exception(Marshal.GetLastWin32Error());
        do
        {
            if (entry.ProcessId != (uint)Environment.ProcessId) continue;
            if (entry.ParentId == 0) throw new IOException("missing parent process");
            // SYNCHRONIZE only, non-inheritable. Never terminate this handle.
            var handle = OpenProcess(0x00100000, false, entry.ParentId);
            if (handle.IsInvalid)
            {
                var error = Marshal.GetLastWin32Error();
                handle.Dispose();
                throw new Win32Exception(error);
            }
            return new ParentLifetime(handle);
        } while (Process32NextW(snapshot, ref entry));
        throw new IOException("parent process not found");
    }

    public void Dispose() { registration.Unregister(null); parent.Dispose(); }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct ProcessEntry
    {
        internal uint Size, Usage, ProcessId;
        internal nuint DefaultHeap;
        internal uint ModuleId, Threads, ParentId;
        internal int BasePriority;
        internal uint Flags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)]
        internal string File;
    }
    private sealed class Snapshot : SafeHandleZeroOrMinusOneIsInvalid
    {
        private Snapshot() : base(true) { }
        protected override bool ReleaseHandle() => CloseHandle(handle);
    }
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern Snapshot CreateToolhelp32Snapshot(uint flags, uint processId);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool Process32FirstW(Snapshot snapshot, ref ProcessEntry entry);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool Process32NextW(Snapshot snapshot, ref ProcessEntry entry);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern SafeWaitHandle OpenProcess(uint access,
        [MarshalAs(UnmanagedType.Bool)] bool inherit, uint processId);
    [DllImport("kernel32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CloseHandle(IntPtr handle);
}
