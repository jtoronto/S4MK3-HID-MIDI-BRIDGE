using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace S4ScreenSupervisor.Tests;

internal static partial class Program
{
    private static ProcessStartInfo FixtureStart(params string[] args)
    {
        var executable = Environment.ProcessPath ?? throw new IOException("missing test executable");
        var start = new ProcessStartInfo(executable);
        if (Path.GetFileNameWithoutExtension(executable) == "dotnet")
            start.ArgumentList.Add(typeof(Program).Assembly.Location);
        start.ArgumentList.Add("--fixture");
        foreach (var arg in args) start.ArgumentList.Add(arg);
        return start;
    }

    private static Process Spawn(params string[] args)
    {
        var start = FixtureStart(args);
        start.UseShellExecute = false;
        start.RedirectStandardInput = start.RedirectStandardOutput = start.RedirectStandardError = true;
        return Process.Start(start) ?? throw new IOException("fixture start failed");
    }

    private static async Task<int> Fixture(string[] args)
    {
        if (args is ["production-owner", var supervisor, var marker])
        {
            var start = new ProcessStartInfo(supervisor);
            foreach (var value in new[] { "--self-test", "connect-stall", "--worker-pid-file", marker, "--assignment-gate" })
                start.ArgumentList.Add(value);
            using var owned = NativeWorker.Start(start, Job.Create());
            await owned.Input.WriteAsync(new byte[] { 0xA5 });
            Console.WriteLine("SUPERVISOR " + owned.Id);
            if (await Console.In.ReadLineAsync() != "DIE") return 88;
            Environment.Exit(73);
        }
        if (args is ["watch-parent"])
        {
            using var watch = ParentLifetime.Open();
            Console.WriteLine("WATCHING");
            await watch.Exited;
            return 0;
        }
        if (args is ["watch-owner"])
        {
            using var watcher = Spawn("watch-parent");
            if (await watcher.StandardOutput.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(5)) != "WATCHING") return 81;
            Console.WriteLine("LEAF " + watcher.Id);
            if (await Console.In.ReadLineAsync() != "DIE") return 82;
            Environment.Exit(73);
        }
        if (args is ["owner", var arm])
        {
            var gate = new byte[1];
            if (await Console.OpenStandardInput().ReadAsync(gate) != 1 || gate[0] != 0xA5) return 87;
            using var leaf = NativeWorker.Start(FixtureStart("worker", "connect-stall"), Job.Create());
            if (arm == "armed")
            {
                await leaf.Input.WriteAsync(new byte[] { 0xA5 });
                if (await ReadLine(leaf.Error) != "ARMED") return 83;
            }
            // The outer observer opens this exact process handle before DIE.
            // NativeWorker retains its own handle and its non-inherited inner job.
            Console.WriteLine("LEAF " + leaf.Id);
            if (await Console.In.ReadLineAsync() != "DIE") return 84;
            Environment.Exit(73);
        }
        if (args is not ["worker", var mode]) return 85;
        return await SelfTest.Worker(mode, null);
    }

    private sealed class ObservedWorker : IWorker
    {
        private readonly NativeWorker worker;
        internal TaskCompletionSource Armed { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        internal TaskCompletionSource FrameSeen { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        internal TaskCompletionSource StopSeen { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        private readonly BytePipe logs = new();
        private readonly Task drain;
        public Stream Input => worker.Input;
        public Stream Output => worker.Output;
        public Stream Error => logs;
        public Task<int> Exited => worker.Exited;
        internal ObservedWorker(NativeWorker worker)
        {
            this.worker = worker;
            drain = Task.Run(async () =>
            {
                try
                {
                    await Wire.ReadReplies(worker.Error, line =>
                    {
                        switch (line)
                        {
                            case "ARMED": Armed.TrySetResult(); break;
                            case "FRAME": FrameSeen.TrySetResult(); break;
                            case "STOP": StopSeen.TrySetResult(); break;
                        }
                    }, default);
                }
                finally { logs.End(); }
            });
        }
        public void Terminate() => worker.Terminate();
        public void Dispose() { worker.Dispose(); logs.Dispose(); }
        internal Task Drained => drain;
    }

    private static async Task NativeTests()
    {
        if (!OperatingSystem.IsWindows())
            throw new PlatformNotSupportedException("Windows fixtures must run on Windows");
        if (!File.Exists(supervisorPath)) throw new FileNotFoundException("publish supervisor and pass --supervisor EXE", supervisorPath);
        foreach (var mode in new[] { "normal", "connect-stall", "send-stall", "stop-stall", "blocked-write" })
            await Test("native owned job contains " + mode, async () =>
            {
                using var worker = new ObservedWorker(NativeWorker.Start(FixtureStart("worker", mode), Job.Create()));
                using var input = new BytePipe();
                using var output = new BytePipe();
                var clock = new ManualTime();
                var run = new Session(worker, input, output, Never(), clock).Run();
                try
                {
                    await worker.Armed.Task.WaitAsync(TimeSpan.FromSeconds(5));
                    if (mode == "connect-stall")
                        await clock.Expire(TimeSpan.FromSeconds(8));
                    else
                    {
                        Check(await ReadLine(output) == "READY", "native READY missing");
                        if (mode is "send-stall" or "blocked-write")
                        {
                            await input.WriteAsync(Frame());
                            if (mode == "send-stall") await worker.FrameSeen.Task.WaitAsync(TimeSpan.FromSeconds(5));
                            await clock.Expire(TimeSpan.FromSeconds(3));
                        }
                        else
                        {
                            if (mode == "normal")
                            {
                                await input.WriteAsync(Frame());
                                Check(await ReadLine(output) == "FRAME_OK", "native frame pair not acknowledged");
                            }
                            await input.WriteAsync(new byte[] { 2,0,0,0,0 });
                            await worker.StopSeen.Task.WaitAsync(TimeSpan.FromSeconds(5));
                            if (mode == "stop-stall") await clock.Expire(TimeSpan.FromMilliseconds(500));
                        }
                    }
                    Check(await run == (mode == "normal" ? 0 : 1), "native outcome misclassified");
                    Check(worker.Exited.IsCompletedSuccessfully, "native worker not reaped");
                    var response = await ReadLine(output);
                    Check(mode == "normal" ? response == "STOPPED" : response.StartsWith("ERROR ", StringComparison.Ordinal),
                        "native failure or STOPPED response missing");
                }
                finally
                {
                    input.End();
                    worker.Terminate();
                    await worker.Exited.WaitAsync(TimeSpan.FromSeconds(2));
                    await run.WaitAsync(TimeSpan.FromSeconds(5));
                    await worker.Drained.WaitAsync(TimeSpan.FromSeconds(2));
                }
            });
        await Test("native stderr pipe flood cannot deadlock stop", async () =>
        {
            using var worker = NativeWorker.Start(FixtureStart("worker", "flood"), Job.Create());
            using var input = new BytePipe();
            using var output = new BytePipe();
            var run = new Session(worker, input, output, Never(), limits: new Budgets { Stop = TimeSpan.FromSeconds(5) }).Run();
            try
            {
                Check(await ReadLine(output) == "READY", "flood fixture not ready");
                await input.WriteAsync(new byte[] { 2,0,0,0,0 });
                Check(await run == 0 && await ReadLine(output) == "STOPPED", "native stderr drain stalled stop");
            }
            finally
            {
                input.End();
                worker.Terminate();
                await worker.Exited.WaitAsync(TimeSpan.FromSeconds(2));
                await run.WaitAsync(TimeSpan.FromSeconds(5));
            }
        });
        await Test("native pre-ARM EOF and wrong byte never connect", async () =>
        {
            foreach (var bad in new[] { false, true })
            {
                using var child = Spawn("worker", "normal");
                var stdout = child.StandardOutput.ReadToEndAsync();
                var stderr = child.StandardError.ReadToEndAsync();
                if (bad) await child.StandardInput.BaseStream.WriteAsync(new byte[] { 0 });
                child.StandardInput.Close();
                await child.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                Check(child.ExitCode == 0 && await stdout == "" && await stderr == "", "connected without ARM");
            }
        });
        await Test("assignment failure kills unarmed worker and reaps exact owned process", async () =>
        {
            // A full job gives real AssignProcessToJobObject failure, rather than
            // injecting a replacement assignment function into production.
            var job = Job.Create();
            var info = new TestLimits { Basic = new TestBasic { Flags = 0x2008, ActiveProcesses = 1 } };
            Check(SetInformationJobObject(job, 9, ref info, (uint)Marshal.SizeOf<TestLimits>()), "cannot configure assignment fixture");
            using var occupant = NativeWorker.Start(FixtureStart("worker", "normal"), job);
            bool failed = false;
            try
            {
                using var unexpected = NativeWorker.Start(FixtureStart("worker", "normal"), job);
            }
            catch (Win32Exception) { failed = true; }
            Check(failed, "full job accepted assignment");
            // Failed Start owns and closes the supplied job, containing occupant too.
            await occupant.Exited.WaitAsync(TimeSpan.FromSeconds(2));
            Check(await ReadLineOrEof(occupant.Output) == "", "assignment failure sent ARM");
        });
        foreach (var arm in new[] { "armed", "unarmed" })
            await Test("nested jobs contain owner death with " + arm + " worker", async () =>
            {
                using var outer = Job.Create();
                using var owner = NativeWorker.Start(FixtureStart("owner", arm), outer);
                using var unrelated = Spawn("worker", "normal");
                Process? leaf = null;
                try
                {
                    // The bridge uses suspended spawn/assign/resume. This private
                    // gate establishes the same precondition for the nesting
                    // fixture: no inner worker exists until outer assignment.
                    await owner.Input.WriteAsync(new byte[] { 0xA5 });
                    var line = await ReadLine(owner.Output);
                    Check(line.StartsWith("LEAF ", StringComparison.Ordinal), "nested assignment failed");
                    leaf = Process.GetProcessById(int.Parse(line[5..]));
                    _ = leaf.SafeHandle;
                    Check(IsProcessInJob(leaf.SafeHandle, outer, out var contained) && contained,
                        "worker broke away from bridge outer job");
                    var died = leaf.WaitForExitAsync();
                    await Line(owner.Input, "DIE");
                    Check(await owner.Exited == 73, "owner fixture did not die deliberately");
                    await died.WaitAsync(TimeSpan.FromSeconds(2));
                    Check(!unrelated.HasExited, "job cleanup touched unrelated child");
                }
                finally
                {
                    outer.Terminate();
                    await owner.Exited.WaitAsync(TimeSpan.FromSeconds(2));
                    if (leaf is not null)
                    {
                        if (!leaf.HasExited) leaf.Kill();
                        await leaf.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                        leaf.Dispose();
                    }
                    unrelated.StandardInput.Close();
                    await unrelated.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                }
            });
        await Test("parent handle detects death without reading stdin", async () =>
        {
            using var owner = Spawn("watch-owner");
            Process? watcher = null;
            try
            {
                var line = await owner.StandardOutput.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(5));
                Check(line is not null && line.StartsWith("LEAF ", StringComparison.Ordinal), "watcher did not subscribe");
                watcher = Process.GetProcessById(int.Parse(line![5..]));
                _ = watcher.SafeHandle;
                var exited = watcher.WaitForExitAsync();
                await owner.StandardInput.WriteLineAsync("DIE");
                await owner.StandardInput.FlushAsync();
                await exited.WaitAsync(TimeSpan.FromSeconds(2));
                Check(watcher.ExitCode == 0, "parent watcher failed");
            }
            finally
            {
                if (!owner.HasExited) owner.Kill();
                await owner.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                if (watcher is not null)
                {
                    if (!watcher.HasExited) watcher.Kill();
                    await watcher.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                    watcher.Dispose();
                }
            }
        });
        await Test("outer owner death kills actual supervisor and its nested stuck worker", async () =>
        {
            var directory = Path.Combine(AppContext.BaseDirectory, "native-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(directory);
            var marker = Path.Combine(directory, "worker.pid");
            var created = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            using var watch = new FileSystemWatcher(directory, "worker.pid");
            watch.Created += (_, _) => created.TrySetResult();
            watch.Renamed += (_, _) => created.TrySetResult();
            watch.EnableRaisingEvents = true;
            using var owner = Spawn("production-owner", supervisorPath, marker);
            Process? supervisor = null, worker = null;
            try
            {
                var line = await owner.StandardOutput.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(5));
                Check(line is not null && line.StartsWith("SUPERVISOR ", StringComparison.Ordinal), "supervisor not assigned");
                supervisor = Process.GetProcessById(int.Parse(line![11..]));
                _ = supervisor.SafeHandle;
                await created.Task.WaitAsync(TimeSpan.FromSeconds(5));
                worker = Process.GetProcessById(int.Parse(await File.ReadAllTextAsync(marker)));
                _ = worker.SafeHandle;
                var supervisorExit = supervisor.WaitForExitAsync();
                var workerExit = worker.WaitForExitAsync();
                Check(!supervisor.HasExited && !worker.HasExited, "fixture died before parent-death test");
                await owner.StandardInput.WriteLineAsync("DIE");
                await owner.StandardInput.FlushAsync();
                await owner.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                await Task.WhenAll(supervisorExit, workerExit).WaitAsync(TimeSpan.FromSeconds(2));
                Check(owner.ExitCode == 73 && supervisor.HasExited && worker.HasExited,
                    "outer job did not contain both production processes");
            }
            finally
            {
                if (!owner.HasExited) owner.Kill();
                await owner.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                foreach (var process in new[] { supervisor, worker })
                {
                    if (process is null) continue;
                    if (!process.HasExited) process.Kill();
                    await process.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(2));
                    process.Dispose();
                }
                watch.EnableRaisingEvents = false;
                if (File.Exists(marker)) File.Delete(marker);
                Directory.Delete(directory);
            }
        });
        Console.WriteLine("Windows fixtures only; no NI, HID, MIDI, screen or service access.");
    }

    private static async Task<string> ReadLineOrEof(Stream stream)
    {
        using var text = new StreamReader(stream, leaveOpen: true);
        return await text.ReadToEndAsync();
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct TestBasic
    {
        internal long ProcessTime, JobTime;
        internal uint Flags;
        internal nuint MinWorkingSet, MaxWorkingSet;
        internal uint ActiveProcesses;
        internal nuint Affinity;
        internal uint Priority, Scheduling;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct TestLimits
    {
        internal TestBasic Basic;
        internal ulong ReadOps, WriteOps, OtherOps, ReadBytes, WriteBytes, OtherBytes;
        internal nuint ProcessMemory, JobMemory, PeakProcessMemory, PeakJobMemory;
    }
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool IsProcessInJob(SafeProcessHandle process, Job job,
        [MarshalAs(UnmanagedType.Bool)] out bool result);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetInformationJobObject(Job job, int infoClass, ref TestLimits limits, uint size);
}
