using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;

// This fixture opens one new ordinary file. It never opens a device or invokes
// a HID API, service, USB collector, NI process or bridge.
public static class WindowsFileFixture
{
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool WriteFile(IntPtr file, byte[] data, uint size,
        out uint written, IntPtr overlapped);

    public static int Main(string[] args)
    {
        if (args.Length != 1) return 2;
        using (FileStream file = new FileStream(args[0], FileMode.CreateNew,
            FileAccess.ReadWrite, FileShare.Read))
        {
            Console.WriteLine("READY " + Process.GetCurrentProcess().Id);
            if (Console.ReadLine() != "WRITE") return 3;
            byte[] motor = new byte[] { 0x31, 1, 0xe0, 0xfe, 0, 0, 1, 0xe0, 0xfe, 0, 0 };
            byte[] positive = new byte[95];
            byte[] prefix = new byte[] { 0x31, 1, 0x20, 1, 0, 0, 1, 0x20, 1, 0x86, 0x29 };
            Array.Copy(prefix, positive, prefix.Length);
            uint count;
            if (!WriteFile(file.SafeFileHandle.DangerousGetHandle(), motor,
                (uint)motor.Length, out count, IntPtr.Zero) || count != motor.Length) return 4;
            if (!WriteFile(file.SafeFileHandle.DangerousGetHandle(), positive,
                (uint)positive.Length, out count, IntPtr.Zero) || count != positive.Length) return 5;
            for (int i = 0; i < 300; ++i)
            {
                motor[2] = (byte)(i & 0xff);
                motor[3] = (byte)(i >> 8);
                if (!WriteFile(file.SafeFileHandle.DangerousGetHandle(), motor,
                    (uint)motor.Length, out count, IntPtr.Zero) || count != motor.Length) return 7;
            }
            Console.WriteLine("WRITTEN");
            if (Console.ReadLine() != "EXIT") return 6;
        }
        return 0;
    }
}
