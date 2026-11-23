namespace CaptureHostFixture
{
    using System;
    using System.ComponentModel;
    using System.IO;
    using System.Runtime.InteropServices;
    using System.Threading;
    using S4UsbCapture;

    public static class Program
    {
        [StructLayout(LayoutKind.Explicit, Size = 20)]
        private struct InputRecord
        {
            [FieldOffset(0)] public ushort EventType;
            [FieldOffset(4)] public int KeyDown;
            [FieldOffset(14)] public char Character;
        }
        [StructLayout(LayoutKind.Sequential)]
        private struct Overlapped
        {
            public IntPtr Internal, InternalHigh;
            public uint Offset, OffsetHigh;
            public IntPtr Event;
        }
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr GetStdHandle(int number);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool WriteFile(IntPtr handle, byte[] buffer, uint length,
            IntPtr written, ref Overlapped overlapped);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool FlushFileBuffers(IntPtr handle);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetOverlappedResult(IntPtr handle, ref Overlapped overlapped,
            out uint transferred, bool wait);
        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
        private static extern bool ReadConsoleInputW(IntPtr handle, out InputRecord record,
            uint length, out uint read);

        private static void WritePipe(byte[] bytes)
        {
            using (EventWaitHandle completion = new EventWaitHandle(false, EventResetMode.ManualReset))
            {
                Overlapped operation = new Overlapped();
                operation.Event = completion.SafeWaitHandle.DangerousGetHandle();
                IntPtr output = GetStdHandle(-11);
                if (!WriteFile(output, bytes, (uint)bytes.Length, IntPtr.Zero, ref operation))
                {
                    int error = Marshal.GetLastWin32Error();
                    if (error != 997) throw new Win32Exception(error);
                    uint written;
                    if (!GetOverlappedResult(output, ref operation, out written, true))
                        throw new Win32Exception(Marshal.GetLastWin32Error());
                    if (written != bytes.Length) throw new IOException("Short pipe write");
                }
                if (!FlushFileBuffers(output)) throw new Win32Exception(Marshal.GetLastWin32Error());
            }
        }

        private static int Fake(string[] args)
        {
            if (args.Length != 10 || args[0] != "-d" || args[2] != "--devices" ||
                args[3] != "3" || args[4] != "-s" || args[6] != "-b" ||
                args[8] != "-o" || args[9] != "-") return 72;
            bool consoleReady = ConsoleAttachment.Prepare();
            string[] scenario = args[1].Split('|');
            byte[] header = new byte[] {
                0xd4, 0xc3, 0xb2, 0xa1, 2, 0, 4, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 16, 0, 249, 0, 0, 0
            };
            if (scenario[0] == "bad") header[0] = 0;
            if (scenario[0] == "bad-snap") Array.Clear(header, 16, 4);
            byte[] first = new byte[12], last = new byte[12];
            Buffer.BlockCopy(header, 0, first, 0, 12);
            Buffer.BlockCopy(header, 12, last, 0, 12);
            WritePipe(first);
            using (EventWaitHandle partial = EventWaitHandle.OpenExisting(scenario[1]))
            using (EventWaitHandle release = EventWaitHandle.OpenExisting(scenario[2]))
            {
                partial.Set();
                if (!release.WaitOne(15000)) return 73;
            }
            WritePipe(last);
            Console.Error.Write("fake-stderr");
            Console.Error.Flush();
            if (scenario[0] == "exit")
            {
                using (EventWaitHandle exit = EventWaitHandle.OpenExisting(scenario[3]))
                    if (!exit.WaitOne(15000)) return 74;
                return 19;
            }
            if (!consoleReady)
            {
                Console.Error.Write(" fixture has no attached console");
                using (EventWaitHandle stopped = new EventWaitHandle(false, EventResetMode.ManualReset))
                    stopped.WaitOne(30000);
                return 75;
            }
            while (true)
            {
                InputRecord record;
                uint read;
                if (!ReadConsoleInputW(GetStdHandle(-10), out record, 1, out read))
                    return 76;
                if (read == 1 && record.EventType == 1 && record.KeyDown != 0 &&
                    record.Character == 'q') break;
            }
            WritePipe(new byte[] { 0x51, 0x52, 0x53 });
            return 0;
        }

        public static int Main(string[] args)
        {
            string diagnostic = Environment.GetEnvironmentVariable("S4_SCREEN_MOTOR_DIAGNOSTIC_CONFIG");
            if (diagnostic != null)
            {
                string folder = Path.GetDirectoryName(diagnostic);
                File.WriteAllText(Path.Combine(folder, "fake-gui-started"), "started");
                int pid = System.Diagnostics.Process.GetCurrentProcess().Id;
                File.WriteAllText(Path.Combine(folder, "capture.metadata.json"), "{\"pid\":" + pid + "}");
                File.WriteAllText(Path.Combine(folder, "capture.readiness.json"),
                    "{\"version\":1,\"launch_id\":\"" + Guid.NewGuid() + "\",\"pid\":" + pid +
                    ",\"case\":\"off\",\"state\":\"ready\",\"receiver_ready\":true,\"case_ready\":true,\"ready\":false,\"right\":null,\"error\":null}");
                string held = Path.Combine(folder, "hold-gui-event.txt");
                if (File.Exists(held))
                    using (EventWaitHandle release = EventWaitHandle.OpenExisting(File.ReadAllText(held)))
                        release.WaitOne(60000);
                return 72;
            }
            if (args.Length == 0 || args[0] != "host") return Fake(args);
            try
            {
                CaptureResult result = CaptureHost.Run(
                    Path.Combine(Path.GetDirectoryName(
                        System.Diagnostics.Process.GetCurrentProcess().MainModule.FileName), "UsbPcapFixture.exe"),
                    args[1], 3, args[2], args[3], args[4], Int32.Parse(args[5]), 20);
                File.WriteAllLines(Path.Combine(args[2], "result.txt"), new string[] {
                    "Success=" + result.Success, "HeaderReady=" + result.HeaderReady,
                    "ForcedTermination=" + result.ForcedTermination,
                    "ParentLost=" + result.ParentLost, "UnexpectedExit=" + result.UnexpectedExit,
                    "ExitCode=" + result.ExitCode, "BytesWritten=" + result.BytesWritten,
                    "UsbPacketLossKnown=" + result.UsbPacketLossKnown,
                    "HasError=" + (result.Error != null)
                });
                if (result.Error != null)
                    File.WriteAllText(Path.Combine(args[2], "result-error.txt"), result.Error);
                return 0;
            }
            catch (Exception error)
            {
                File.WriteAllText(Path.Combine(args[2], "helper-error.txt"), error.ToString());
                return 90;
            }
        }
    }
}
