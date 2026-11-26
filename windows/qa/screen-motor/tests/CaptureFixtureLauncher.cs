using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
public static class CaptureFixtureLauncher
{
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct StartupInfo
    {
        public int cb;
        public string reserved, desktop, title;
        public int x, y, xSize, ySize, xChars, yChars, fill, flags;
        public short show, reserved2;
        public IntPtr reservedPtr, stdin, stdout, stderr;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct ProcessInformation
    {
        public IntPtr process, thread;
        public int processId, threadId;
    }
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern bool CreateProcessW(string application, StringBuilder command,
        IntPtr processAttributes, IntPtr threadAttributes, bool inherit, uint flags,
        IntPtr environment, string directory, ref StartupInfo startup, out ProcessInformation info);
    [DllImport("kernel32.dll")]
    private static extern bool CloseHandle(IntPtr handle);
    public static Process Start(string executable, string arguments)
    {
        StartupInfo startup = new StartupInfo();
        startup.cb = Marshal.SizeOf(typeof(StartupInfo));
        ProcessInformation info;
        StringBuilder command = new StringBuilder("\"" + executable + "\" " + arguments);
        if (!CreateProcessW(executable, command, IntPtr.Zero, IntPtr.Zero, false, 0x10,
            IntPtr.Zero, null, ref startup, out info))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        try { return Process.GetProcessById(info.processId); }
        finally { CloseHandle(info.thread); CloseHandle(info.process); }
    }
}
