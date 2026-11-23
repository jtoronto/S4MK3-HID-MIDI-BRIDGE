namespace S4UsbCapture
{
    using System;
    using System.ComponentModel;
    using System.Runtime.InteropServices;
    using System.Text;

    internal sealed class CaptureConsole
    {
        [StructLayout(LayoutKind.Sequential)]
        private struct Coordinate { public short X, Y; }
        [StructLayout(LayoutKind.Sequential)]
        private struct Rectangle { public short Left, Top, Right, Bottom; }
        [StructLayout(LayoutKind.Sequential)]
        private struct BufferInfo
        {
            public Coordinate Size, Cursor;
            public ushort Attributes;
            public Rectangle Window;
            public Coordinate Maximum;
        }
        [StructLayout(LayoutKind.Explicit, Size = 20)]
        private struct InputRecord
        {
            [FieldOffset(0)] public ushort EventType;
            [FieldOffset(4)] public int KeyDown;
            [FieldOffset(8)] public ushort RepeatCount;
            [FieldOffset(10)] public ushort VirtualKeyCode;
            [FieldOffset(14)] public char UnicodeChar;
        }
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr GetStdHandle(int number);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetConsoleMode(IntPtr handle, out uint mode);
        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
        private static extern bool WriteConsoleInputW(IntPtr handle, ref InputRecord record, uint count, out uint written);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetConsoleScreenBufferInfo(IntPtr handle, out BufferInfo information);
        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
        private static extern bool ReadConsoleOutputCharacterW(IntPtr handle, StringBuilder text,
            uint count, Coordinate start, out uint read);

        private readonly IntPtr input, output;
        private CaptureConsole(IntPtr input, IntPtr output) { this.input = input; this.output = output; }

        internal static CaptureConsole Open()
        {
            IntPtr input = GetStdHandle(-10), output = GetStdHandle(-12);
            uint mode;
            if (!GetConsoleMode(input, out mode) || !GetConsoleMode(output, out mode))
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Capture host requires console input and stderr");
            return new CaptureConsole(input, output);
        }

        internal void Quit()
        {
            InputRecord key = new InputRecord { EventType = 1, KeyDown = 1, RepeatCount = 1,
                VirtualKeyCode = 0x51, UnicodeChar = 'q' };
            uint written;
            if (!WriteConsoleInputW(input, ref key, 1, out written) || written != 1)
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not send console stop");
        }

        internal string Tail()
        {
            BufferInfo info;
            if (!GetConsoleScreenBufferInfo(output, out info))
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read capture console");
            int rows = Math.Min(info.Cursor.Y + 1, 512);
            int count = checked(info.Size.X * rows);
            StringBuilder text = new StringBuilder(count + 1);
            uint read;
            Coordinate start = new Coordinate { X = 0, Y = checked((short)(info.Cursor.Y + 1 - rows)) };
            if (!ReadConsoleOutputCharacterW(output, text, checked((uint)count), start, out read))
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not preserve capture console");
            string raw = text.ToString(0, checked((int)read));
            StringBuilder lines = new StringBuilder();
            for (int offset = 0; offset < raw.Length; offset += info.Size.X)
                lines.AppendLine(raw.Substring(offset, Math.Min(info.Size.X, raw.Length - offset)).TrimEnd());
            return lines.ToString().TrimEnd();
        }
    }
}
