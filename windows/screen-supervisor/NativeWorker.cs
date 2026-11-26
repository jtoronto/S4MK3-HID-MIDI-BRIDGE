using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace S4ScreenSupervisor;

internal sealed class NativeWorker : IWorker
{
    private readonly Process process;
    private readonly Job job;
    internal int Id => process.Id;
    public Stream Input => process.StandardInput.BaseStream;
    public Stream Output => process.StandardOutput.BaseStream;
    public Stream Error => process.StandardError.BaseStream;
    public Task<int> Exited { get; }

    private NativeWorker(Process process, Job job)
    {
        this.process = process;
        this.job = job;
        Exited = Reap();
    }

    private async Task<int> Reap()
    {
        await process.WaitForExitAsync();
        return process.ExitCode;
    }

    // Takes ownership of the job on entry, including every failure path.
    internal static NativeWorker Start(ProcessStartInfo start, Job job)
    {
        var process = new Process { StartInfo = start };
        bool started = false;
        try
        {
            start.UseShellExecute = false;
            start.CreateNoWindow = true;
            start.RedirectStandardInput = start.RedirectStandardOutput = start.RedirectStandardError = true;
            if (!process.Start()) throw new IOException("worker start failed");
            started = true;
            // Opening the exact handle also prevents PID reuse during cleanup.
            _ = process.SafeHandle;
            job.Assign(process);
            return new NativeWorker(process, job);
        }
        catch
        {
            job.Dispose();
            if (started)
            {
                // The unassigned worker has never received ARM. Kill only this
                // retained process, never a PID/name/service or the bridge.
                if (!process.HasExited) process.Kill();
                if (!process.WaitForExit(2000)) throw new IOException("unassigned worker reap deadline");
            }
            process.Dispose();
            throw;
        }
    }

    public void Terminate() => job.Terminate();
    public void Dispose()
    {
        job.Dispose();
        process.Dispose();
    }
}

internal sealed class Job : SafeHandleZeroOrMinusOneIsInvalid
{
    private Job() : base(true) { }
    internal static Job Create()
    {
        // Unnamed, null SECURITY_ATTRIBUTES: never inherited by the worker.
        var job = CreateJobObjectW(IntPtr.Zero, null);
        if (job.IsInvalid)
        {
            var error = Marshal.GetLastWin32Error();
            job.Dispose();
            throw new Win32Exception(error);
        }
        var info = new ExtendedLimits { Basic = new BasicLimits { Flags = 0x2000 } };
        if (!SetInformationJobObject(job, 9, ref info, (uint)Marshal.SizeOf<ExtendedLimits>()))
        {
            var error = Marshal.GetLastWin32Error();
            job.Dispose();
            throw new Win32Exception(error);
        }
        return job;
    }

    internal void Assign(Process child)
    {
        if (!AssignProcessToJobObject(this, child.SafeHandle))
            throw new Win32Exception(Marshal.GetLastWin32Error());
    }
    internal void Terminate()
    {
        if (!TerminateJobObject(this, 125))
        {
            var error = Marshal.GetLastWin32Error();
            Dispose(); // Kill-on-close remains the fallback if explicit termination fails.
            throw new Win32Exception(error);
        }
    }
    protected override bool ReleaseHandle() => CloseHandle(handle);

    [StructLayout(LayoutKind.Sequential)]
    private struct BasicLimits
    {
        internal long ProcessTime, JobTime;
        internal uint Flags;
        internal nuint MinimumWorkingSet, MaximumWorkingSet;
        internal uint ActiveProcesses;
        internal nuint Affinity;
        internal uint Priority, Scheduling;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct IoCounters
    {
        internal ulong ReadOperations, WriteOperations, OtherOperations, ReadBytes, WriteBytes, OtherBytes;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct ExtendedLimits
    {
        internal BasicLimits Basic;
        internal IoCounters Io;
        internal nuint ProcessMemory, JobMemory, PeakProcessMemory, PeakJobMemory;
    }

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern Job CreateJobObjectW(IntPtr attributes, string? name);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetInformationJobObject(Job job, int infoClass, ref ExtendedLimits info, uint size);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool AssignProcessToJobObject(Job job, SafeProcessHandle process);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool TerminateJobObject(Job job, uint exitCode);
    [DllImport("kernel32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CloseHandle(IntPtr handle);
}
