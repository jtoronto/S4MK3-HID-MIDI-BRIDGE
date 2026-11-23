// USBPcapCMD 1.5.4.0's GUI entry point attaches to its parent console only
// when at least one output stream is not redirected. No device is opened.
namespace CaptureHostFixture
{
    using System;
    using System.Runtime.InteropServices;

    internal static class ConsoleAttachment
    {
        [StructLayout(LayoutKind.Sequential)]
        private struct FileInformation
        {
            public uint Attributes, CreationLow, CreationHigh, AccessLow, AccessHigh;
            public uint WriteLow, WriteHigh, Volume, SizeHigh, SizeLow, Links, IndexHigh, IndexLow;
        }

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr GetStdHandle(int number);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool SetStdHandle(int number, IntPtr handle);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetFileInformationByHandle(IntPtr handle, out FileInformation information);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool AttachConsole(uint processId);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetConsoleMode(IntPtr handle, out uint mode);

        internal static bool Prepare()
        {
            IntPtr output = GetStdHandle(-11), error = GetStdHandle(-12);
            FileInformation information;
            bool outputRedirected = GetFileInformationByHandle(output, out information);
            bool errorRedirected = GetFileInformationByHandle(error, out information);
            if (!(outputRedirected && errorRedirected) && AttachConsole(UInt32.MaxValue))
            {
                if (outputRedirected && !SetStdHandle(-11, output))
                    throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
                if (errorRedirected && !SetStdHandle(-12, error))
                    throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
            }
            uint mode;
            return GetConsoleMode(GetStdHandle(-10), out mode);
        }
    }
}
