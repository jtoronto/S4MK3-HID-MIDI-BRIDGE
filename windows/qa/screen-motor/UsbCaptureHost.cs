using System;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Threading;

namespace S4UsbCapture
{

    public static class CaptureHost
    {
        private const int StopTimeoutMs = 10000;

        private static string Quote(string value)
        {
            StringBuilder result = new StringBuilder("\"");
            int slashes = 0;
            foreach (char c in value)
            {
                if (c == '\\') { slashes++; continue; }
                if (c == '"') result.Append('\\', slashes * 2 + 1);
                else result.Append('\\', slashes);
                result.Append(c);
                slashes = 0;
            }
            result.Append('\\', slashes * 2);
            return result.Append('"').ToString();
        }

        private static bool ValidHeader(byte[] header)
        {
            return header[0] == 0xd4 && header[1] == 0xc3 &&
                header[2] == 0xb2 && header[3] == 0xa1 &&
                header[4] == 2 && header[5] == 0 &&
                header[6] == 4 && header[7] == 0 &&
                BitConverter.ToUInt32(header, 16) != 0 &&
                header[20] == 249 && header[21] == 0 &&
                header[22] == 0 && header[23] == 0;
        }

        private static void SignalIfOpen(EventWaitHandle signal)
        {
            try { signal.Set(); }
            catch (ObjectDisposedException) { }
        }

        public static bool WaitForReady(EventWaitHandle ready, Process host, int timeoutMs)
        {
            using (ManualResetEvent exited = new ManualResetEvent(false))
            {
                EventHandler handler = delegate { SignalIfOpen(exited); };
                host.Exited += handler;
                try
                {
                    host.EnableRaisingEvents = true;
                    if (host.HasExited) return false;
                    int selected = WaitHandle.WaitAny(new WaitHandle[] { exited, ready }, timeoutMs);
                    return selected == 1 && !host.HasExited;
                }
                finally { host.Exited -= handler; }
            }
        }

        public static CaptureResult Run(string exe, string captureInterface, int deviceAddress,
            string directory, string readyEventName, string stopEventName, int parentPid, int maximumSeconds)
        {
            CaptureResult result = new CaptureResult { StartedUtc = DateTime.UtcNow, ExitCode = -1 };
            Process child = null;
            bool started = false;
            Thread outputThread = null;
            Exception streamError = null;
            object errorLock = new object();
            long bytes = 0;
            try
            {
                if (maximumSeconds < 1 || maximumSeconds > 300)
                    throw new ArgumentOutOfRangeException("maximumSeconds");
                if (deviceAddress < 1 || deviceAddress > 127)
                    throw new ArgumentOutOfRangeException("deviceAddress");
                CaptureConsole console = CaptureConsole.Open();

                Process parent;
                try { parent = Process.GetProcessById(parentPid); }
                catch (ArgumentException)
                {
                    result.ParentLost = true;
                    throw new InvalidOperationException("Parent process does not exist");
                }
                using (EventWaitHandle ready = EventWaitHandle.OpenExisting(readyEventName))
                using (EventWaitHandle stop = EventWaitHandle.OpenExisting(stopEventName))
                using (ManualResetEvent exited = new ManualResetEvent(false))
                using (ManualResetEvent parentExited = new ManualResetEvent(false))
                using (ManualResetEvent streamFailed = new ManualResetEvent(false))
                using (parent)
                using (FileStream capture = new FileStream(Path.Combine(directory, "capture.pcap"),
                    FileMode.CreateNew, FileAccess.Write, FileShare.Read))
                using (FileStream errors = new FileStream(Path.Combine(directory, "usb-stderr.txt"),
                    FileMode.CreateNew, FileAccess.Write, FileShare.Read))
                {
                    try
                    {
                    parent.Exited += delegate { SignalIfOpen(parentExited); };
                    parent.EnableRaisingEvents = true;
                    if (parent.HasExited)
                    {
                        result.ParentLost = true;
                        throw new InvalidOperationException("Parent process already exited");
                    }
                    child = new Process();
                    child.StartInfo = new ProcessStartInfo {
                        FileName = exe,
                        Arguments = "-d " + Quote(captureInterface) + " --devices " + deviceAddress +
                            " -s 1048576 -b 16777216 -o -",
                        UseShellExecute = false,
                        RedirectStandardOutput = true,
                        // Stock USBPcapCMD skips AttachConsole when both outputs
                        // are redirected. Keep stderr on this isolated console.
                        RedirectStandardError = false,
                        CreateNoWindow = false
                    };
                    child.Exited += delegate { SignalIfOpen(exited); };
                    child.EnableRaisingEvents = true;
                    if (!child.Start()) throw new InvalidOperationException("Capture process did not start");
                    started = true;

                    outputThread = new Thread(delegate()
                    {
                        try
                        {
                            byte[] buffer = new byte[65536];
                            byte[] header = new byte[24];
                            int headerCount = 0;
                            int count;
                            Stream input = child.StandardOutput.BaseStream;
                            while ((count = input.Read(buffer, 0, buffer.Length)) != 0)
                            {
                                capture.Write(buffer, 0, count);
                                Interlocked.Add(ref bytes, count);
                                if (headerCount < header.Length)
                                {
                                    int copy = Math.Min(count, header.Length - headerCount);
                                    Buffer.BlockCopy(buffer, 0, header, headerCount, copy);
                                    headerCount += copy;
                                    if (headerCount == header.Length)
                                    {
                                        if (!ValidHeader(header)) throw new InvalidDataException("Invalid PCAP header");
                                        capture.Flush(true);
                                        if (child.HasExited) throw new InvalidDataException("Capture exited before readiness");
                                        if (parentExited.WaitOne(0))
                                            throw new InvalidOperationException("Parent exited before readiness");
                                        result.HeaderReady = true;
                                        ready.Set();
                                    }
                                }
                            }
                            if (headerCount < header.Length) throw new EndOfStreamException("Incomplete PCAP header");
                            capture.Flush(true);
                        }
                        catch (Exception ex)
                        {
                            lock (errorLock) { if (streamError == null) streamError = ex; }
                            SignalIfOpen(streamFailed);
                        }
                    });
                    outputThread.IsBackground = true;
                    outputThread.Start();

                    int selected = WaitHandle.WaitAny(new WaitHandle[] { exited, streamFailed, parentExited, stop },
                        checked(maximumSeconds * 1000));
                    if (selected == 0) { result.UnexpectedExit = true; throw new InvalidOperationException("Capture exited unexpectedly"); }
                    if (selected == 1) throw new IOException("Capture stream failed");
                    if (selected == 2) { result.ParentLost = true; throw new InvalidOperationException("Parent process exited"); }
                    if (selected == WaitHandle.WaitTimeout) throw new TimeoutException("Capture duration expired");
                    if (!result.HeaderReady) throw new InvalidOperationException("Stop requested before capture readiness");
                    if (child.HasExited)
                    {
                        result.UnexpectedExit = true;
                        throw new InvalidOperationException("Capture exited before console stop");
                    }

                    console.Quit();
                    int stopped = WaitHandle.WaitAny(new WaitHandle[] { exited, streamFailed, parentExited }, StopTimeoutMs);
                    if (stopped == 1) throw new IOException("Capture stream failed while stopping");
                    if (stopped == 2) { result.ParentLost = true; throw new InvalidOperationException("Parent process exited"); }
                    if (stopped == WaitHandle.WaitTimeout) throw new TimeoutException("Capture did not stop");
                    if (!outputThread.Join(StopTimeoutMs))
                        throw new TimeoutException("Capture streams did not finish");
                    lock (errorLock) { if (streamError != null) throw new IOException("Capture stream failed", streamError); }
                    result.ExitCode = child.ExitCode;
                    if (result.ExitCode != 0) throw new InvalidOperationException("Capture exited with code " + result.ExitCode);
                    result.Success = true;
                    }
                    finally
                    {
                        if (parentExited.WaitOne(0))
                        {
                            result.ParentLost = true;
                            result.Success = false;
                            if (result.Error == null) result.Error = "Parent process exited";
                        }
                        if (started)
                        {
                            if (!child.HasExited)
                            {
                                result.ForcedTermination = true;
                                child.Kill();
                                if (!child.WaitForExit(StopTimeoutMs))
                                    throw new TimeoutException("Capture did not exit after termination");
                            }
                            if (outputThread != null && !outputThread.Join(StopTimeoutMs))
                                throw new TimeoutException("Capture output thread did not finish");
                            result.ExitCode = child.ExitCode;
                        }
                        byte[] transcript = Encoding.UTF8.GetBytes(console.Tail());
                        errors.Write(transcript, 0, transcript.Length);
                        errors.Flush(true);
                    }
                }
            }
            catch (Exception ex)
            {
                result.Error = ex.ToString();
                lock (errorLock) { if (streamError != null) result.Error += Environment.NewLine + streamError; }
            }
            finally
            {
                if (started)
                {
                    try
                    {
                        if (!child.HasExited)
                        {
                            result.ForcedTermination = true;
                            child.Kill();
                            if (!child.WaitForExit(StopTimeoutMs))
                                result.Error += Environment.NewLine + "Capture process did not exit after termination";
                        }
                        if (child.HasExited) result.ExitCode = child.ExitCode;
                    }
                    catch (Exception ex) { result.Error += Environment.NewLine + "Capture cleanup failed: " + ex; }
                }
                if (child != null) child.Dispose();
                if (result.Error != null) result.Success = false;
                result.BytesWritten = Interlocked.Read(ref bytes);
                result.EndedUtc = DateTime.UtcNow;
            }
            return result;
        }
    }
}
