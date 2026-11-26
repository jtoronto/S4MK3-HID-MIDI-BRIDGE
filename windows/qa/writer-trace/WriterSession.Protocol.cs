using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Runtime.Serialization;
using System.Runtime.Serialization.Json;
using System.Text;
using System.Threading;
using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

public sealed partial class WriterSession
{
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool PeekNamedPipe(SafeFileHandle pipe, IntPtr buffer,
        uint size, IntPtr read, out uint available, IntPtr left);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool ReadFile(SafeFileHandle file, byte[] buffer,
        uint size, out uint read, IntPtr overlapped);

    private int ReadOutput(Stream input, byte[] buffer)
    {
        if (Environment.OSVersion.Platform != PlatformID.Win32NT)
            return input.Read(buffer, 0, buffer.Length);
        FileStream pipe = (FileStream)input;
        while (true)
        {
            // Observe producer exit before peeking: its final writes may land
            // between an empty peek and the exit notification.
            bool exited = process.HasExited;
            uint available;
            if (!PeekNamedPipe(pipe.SafeFileHandle, IntPtr.Zero, 0,
                IntPtr.Zero, out available, IntPtr.Zero))
            {
                int error = Marshal.GetLastWin32Error();
                if (error == 109) return 0; // ERROR_BROKEN_PIPE
                throw new Win32Exception(error);
            }
            if (available != 0)
            {
                uint read;
                // Read directly so FileStream read-ahead cannot hide queued
                // bytes from the following native availability check.
                if (!ReadFile(pipe.SafeFileHandle, buffer,
                    Math.Min(available, (uint)buffer.Length), out read, IntPtr.Zero))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                return (int)read;
            }
            // Other processes can retain pipe writers after the injector exits.
            // All bytes from our producer are now queued and drained.
            if (exited) return 0;
            process.WaitForExit(20);
        }
    }

    private void Drain(Stream input, string path, bool stdout)
    {
        try
        {
            using (FileStream output = new FileStream(path, FileMode.Append, FileAccess.Write, FileShare.Read))
            using (MemoryStream line = new MemoryStream())
            {
                byte[] buffer = new byte[4096];
                int count;
                while ((count = ReadOutput(input, buffer)) != 0)
                {
                    output.Write(buffer, 0, count);
                    output.Flush();
                    if (!stdout) continue;
                    for (int i = 0; i < count; i++)
                    {
                        if (buffer[i] == 10)
                        {
                            ParseLine(line.ToArray());
                            line.SetLength(0);
                        }
                        else line.WriteByte(buffer[i]);
                    }
                }
                if (stdout)
                {
                    if (line.Length != 0) ParseLine(line.ToArray());
                    lock (gate)
                    {
                        if (!stopped && !abandoned) Fail(new EndOfStreamException("Injector stdout closed before the stopped acknowledgement."));
                    }
                }
            }
        }
        catch (Exception ex)
        {
            lock (gate)
            {
                if (drainFailure == null) drainFailure = ex;
                if (!abandoned) Fail(ex);
            }
        }
    }

    private void ParseLine(byte[] bytes)
    {
        Message message;
        try
        {
            using (MemoryStream stream = new MemoryStream(bytes))
                message = (Message)new DataContractJsonSerializer(typeof(Message)).ReadObject(stream);
        }
        catch (SerializationException) { return; }
        catch (System.Xml.XmlException) { return; }
        if (message == null || message.Kind == null) return;
        lock (gate)
        {
            if (message.Kind == "error")
                Fail(new InvalidOperationException("Agent reported an error."));
            else if (message.Kind == "ready" || message.Kind == "batch" ||
                     message.Kind == "summary" || message.Kind == "stopped")
            {
                if (message.Pid != targetPid)
                    Fail(new InvalidOperationException("Agent message has the wrong PID."));
                else if (message.Kind == "ready")
                    ready = true;
                else if (message.Kind == "batch")
                {
                    if (message.Events == null || !message.LastSeq.HasValue)
                        Fail(new InvalidOperationException("Invalid agent batch."));
                    else
                        foreach (AgentEvent item in message.Events)
                            if (item == null || (item.Kind != "entry" && item.Kind != "return" &&
                                                 item.Kind != "stack"))
                                Fail(new InvalidOperationException("Agent batch contains an invalid or error event."));
                }
                else if (message.Kind == "stopped")
                {
                    if (message.Reason == "operator" || message.Reason == "deadline")
                        stopped = true;
                    if (!stopping || message.Reason != "operator" || message.HookCount != 0 ||
                        !CleanCounters(message))
                        Fail(new InvalidOperationException("Invalid or incomplete stopped acknowledgement."));
                }
                else if (!CleanCounters(message))
                    Fail(new InvalidOperationException("Agent reported incomplete collection."));
            }
            Monitor.PulseAll(gate);
        }
    }

    private static bool CleanCounters(Message message)
    {
        return message.Dropped.HasValue && message.Errors.HasValue &&
               message.OutstandingCalls.HasValue && message.Dropped.Value == 0 &&
               message.Errors.Value == 0 && message.OutstandingCalls.Value == 0;
    }

}
