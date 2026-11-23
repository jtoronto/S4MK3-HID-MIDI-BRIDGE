using System;
using System.Diagnostics;
using System.IO;
using System.Text;

// Compile together with WriterSession.cs as a console executable/apphost.
// The same executable acts as a harmless injector fixture.
public static class WriterSessionTests
{
    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(10);

    public static int Main(string[] args)
    {
        if (args.Length > 0 && args[0] == "--interactive") return Fixture(args);
        string executable = args.Length == 1 ? args[0] : Process.GetCurrentProcess().MainModule.FileName;
        string root = Path.Combine(Path.GetTempPath(), "writer-session-tests-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(root);
        try
        {
            DistinctEvidencePaths(executable, root);
            HappyPath(executable, root);
            RejectReady(executable, root, "wrong-ready");
            RejectReady(executable, root, "error");
            RejectReady(executable, root, "eof");
            RejectReady(executable, root, "batch-error");
            RejectReady(executable, root, "batch-missing-seq");
            RejectReady(executable, root, "summary-dropped");
            FailureDrainsEvidence(executable, root);
            DeadlineAlreadyStopped(executable, root);
            RejectStop(executable, root, "wrong-stopped");
            RejectStop(executable, root, "hooks");
            RejectStop(executable, root, "missing-reason");
            RejectStop(executable, root, "deadline");
            RejectStop(executable, root, "missing-counters");
            RejectStop(executable, root, "outstanding");
            RejectStop(executable, root, "stop-error");
            RejectStop(executable, root, "exit-failure");
            HealthFailure(executable, root);
            ReadyTimeout(executable, root);
            StopTimeout(executable, root);
            ExitTimeout(executable, root);
            Console.WriteLine("WriterSession tests passed.");
            return 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine(ex);
            Console.Error.WriteLine("Evidence retained at " + root);
            return 1;
        }
    }

    private static void DistinctEvidencePaths(string executable, string root)
    {
        using (WriterSession first = new WriterSession(executable, 12345, "success", root))
        using (WriterSession second = new WriterSession(executable, 12345, "success", root))
        {
            if (first.StdoutPath == second.StdoutPath || first.StderrPath == second.StderrPath)
                throw new Exception("Sessions must use distinct evidence paths.");
        }
    }

    private static void HappyPath(string executable, string root)
    {
        using (WriterSession session = new WriterSession(executable, 12345, "success", root))
        {
            session.Start(Timeout);
            session.ThrowIfFailed();
            try { session.Mark("invalid\nSTOP"); throw new Exception("Multiline marker accepted."); }
            catch (ArgumentException) { }
            session.Mark("fixture marker");
            session.Stop(Timeout);
            byte[] expected = Encoding.UTF8.GetBytes("{\"kind\":\"ready\",\"pid\":12345}\n{\"kind\":\"batch\",\"pid\":12345,\"events\":[{\"kind\":\"entry\",\"data\":\"\u00e9\"}],\"last_seq\":1}\r\n{\"kind\":\"stopped\",\"pid\":12345,\"reason\":\"operator\",\"hook_count\":0,\"dropped\":0,\"errors\":0,\"outstanding_calls\":0}\n");
            Equal(expected, File.ReadAllBytes(session.StdoutPath), "stdout bytes");
            Equal(new byte[] { 0xff, 0x00, 0x0a }, File.ReadAllBytes(session.StderrPath), "stderr bytes");
        }
    }

    private static void RejectReady(string executable, string root, string scenario)
    {
        using (WriterSession session = new WriterSession(executable, 12345, scenario, root))
        {
            ExpectFailure(delegate { session.Start(Timeout); }, scenario);
            if (File.ReadAllBytes(session.StdoutPath).Length == 0)
                throw new Exception("Missing failure evidence for " + scenario);
        }
    }

    private static void RejectStop(string executable, string root, string scenario)
    {
        using (WriterSession session = new WriterSession(executable, 12345, scenario, root))
        {
            session.Start(Timeout);
            ExpectFailure(delegate { session.Stop(Timeout); }, scenario);
            if (File.ReadAllBytes(session.StdoutPath).Length == 0)
                throw new Exception("Missing failure evidence for " + scenario);
        }
    }

    private static void FailureDrainsEvidence(string executable, string root)
    {
        using (WriterSession session = new WriterSession(executable, 12345, "failure-drain", root))
        {
            ExpectFailure(delegate { session.Start(Timeout); }, "failure-drain");
            byte[] expected = new byte[65536];
            for (int i = 0; i < expected.Length; i++) expected[i] = (byte)(i % 251);
            Equal(expected, File.ReadAllBytes(session.StderrPath), "failure stderr");
        }
    }

    private static void DeadlineAlreadyStopped(string executable, string root)
    {
        using (WriterSession session = new WriterSession(executable, 12345, "deadline-early", root))
        {
            ExpectFailure(delegate { session.Start(Timeout); }, "deadline-early");
            Equal(Encoding.UTF8.GetBytes("deadline closed"), File.ReadAllBytes(session.StderrPath), "deadline cleanup");
        }
    }

    private static void ReadyTimeout(string executable, string root)
    {
        using (WriterSession session = new WriterSession(executable, 12345, "wait", root))
        {
            try { session.Start(TimeSpan.FromMilliseconds(250)); }
            catch (TimeoutException) { return; }
            throw new Exception("Ready timeout was not enforced.");
        }
    }

    private static void HealthFailure(string executable, string root)
    {
        using (WriterSession session = new WriterSession(executable, 12345, "health", root))
        {
            session.Start(Timeout);
            session.Mark("trigger");
            ExpectFailure(delegate { session.Stop(Timeout); }, "health");
            Equal(Encoding.UTF8.GetBytes("graceful exit"), File.ReadAllBytes(session.StderrPath), "graceful failure cleanup");
            try { session.ThrowIfFailed(); }
            catch (InvalidOperationException ex)
            {
                if (ex.InnerException != null) return;
                throw;
            }
            throw new Exception("Asynchronous agent failure was not reported.");
        }
    }

    private static void StopTimeout(string executable, string root)
    {
        using (WriterSession session = new WriterSession(executable, 12345, "stop-wait", root))
        {
            session.Start(Timeout);
            try { session.Stop(TimeSpan.FromMilliseconds(250)); }
            catch (TimeoutException) { return; }
            throw new Exception("Stop timeout was not enforced.");
        }
    }

    private static void ExitTimeout(string executable, string root)
    {
        using (WriterSession session = new WriterSession(executable, 12345, "exit-wait", root))
        {
            session.Start(Timeout);
            try { session.Stop(TimeSpan.FromMilliseconds(250)); }
            catch (TimeoutException) { return; }
            throw new Exception("Exit timeout was not enforced.");
        }
    }

    private static void ExpectFailure(Action action, string scenario)
    {
        try { action(); }
        catch (InvalidOperationException) { return; }
        throw new Exception("Expected failure for " + scenario);
    }

    private static void Equal(byte[] expected, byte[] actual, string name)
    {
        if (expected.Length != actual.Length) throw new Exception(name + " length differs.");
        for (int i = 0; i < expected.Length; i++)
            if (expected[i] != actual[i]) throw new Exception(name + " differs at byte " + i + ".");
    }

    private static int Fixture(string[] args)
    {
        if (args.Length != 6 || args[1] != "--runtime=qjs" || args[2] != "-p" || args[4] != "-s") return 2;
        string pid = args[3];
        string scenario = args[5];
        Stream stdout = Console.OpenStandardOutput();
        Stream stderr = Console.OpenStandardError();
        if (scenario == "deadline-early")
        {
            Write(stdout, "{\"kind\":\"stopped\",\"pid\":" + pid + ",\"reason\":\"deadline\",\"hook_count\":0,\"dropped\":0,\"errors\":0,\"outstanding_calls\":0}\n");
            if (Console.ReadLine() != null) return 9;
            Write(stderr, "deadline closed");
            return 0;
        }
        if (scenario == "failure-drain")
        {
            byte[] evidence = new byte[65536];
            for (int i = 0; i < evidence.Length; i++) evidence[i] = (byte)(i % 251);
            stderr.Write(evidence, 0, evidence.Length);
            stderr.Flush();
            Write(stdout, "{\"kind\":\"error\",\"pid\":" + pid + "}\n");
            if (Console.ReadLine() != "STOP") return 7;
            Write(stdout, "{\"kind\":\"stopped\",\"pid\":" + pid + ",\"reason\":\"operator\",\"hook_count\":0,\"dropped\":0,\"errors\":0,\"outstanding_calls\":0}\n");
            return Console.ReadLine() == null ? 0 : 8;
        }
        if (scenario == "wait") { Console.ReadLine(); return 0; }
        if (scenario == "error") { Write(stdout, "{\"kind\":\"error\",\"pid\":" + pid + "}\n"); return 0; }
        if (scenario == "eof") { Write(stdout, "fixture closed\n"); return 0; }
        if (scenario == "wrong-ready") { Write(stdout, "{\"kind\":\"ready\",\"pid\":999}\n"); return 0; }
        if (scenario == "batch-error") { Write(stdout, "{\"kind\":\"batch\",\"pid\":" + pid + ",\"events\":[{\"kind\":\"error\"}],\"last_seq\":1}\n"); return 0; }
        if (scenario == "batch-missing-seq") { Write(stdout, "{\"kind\":\"batch\",\"pid\":" + pid + ",\"events\":[]}\n"); return 0; }
        if (scenario == "summary-dropped") { Write(stdout, "{\"kind\":\"summary\",\"pid\":" + pid + ",\"dropped\":1,\"errors\":0,\"outstanding_calls\":0}\n"); return 0; }
        Write(stdout, "{\"kind\":\"ready\",\"pid\":" + pid + "}\n");
        if (scenario == "success")
        {
            if (Console.ReadLine() != "MARK fixture marker") return 6;
            Write(stdout, "{\"kind\":\"batch\",\"pid\":" + pid + ",\"events\":[{\"kind\":\"entry\",\"data\":\"\u00e9\"}],\"last_seq\":1}\r\n");
            stderr.Write(new byte[] { 0xff, 0x00, 0x0a }, 0, 3);
            stderr.Flush();
        }
        if (scenario == "health")
        {
            if (Console.ReadLine() != "MARK trigger") return 6;
            Write(stdout, "{\"kind\":\"batch\",\"pid\":" + pid + ",\"events\":[{\"kind\":\"error\"}],\"last_seq\":1}\n");
            if (Console.ReadLine() != "STOP") return 7;
            Write(stdout, "{\"kind\":\"stopped\",\"pid\":" + pid + ",\"reason\":\"operator\",\"hook_count\":0,\"dropped\":0,\"errors\":0,\"outstanding_calls\":0}\n");
            if (Console.ReadLine() != null) return 8;
            Write(stderr, "graceful exit");
            return 0;
        }
        if (Console.ReadLine() != "STOP") return 3;
        if (scenario == "stop-wait") { Console.ReadLine(); return 0; }
        if (scenario == "stop-error") { Write(stdout, "{\"kind\":\"error\",\"pid\":" + pid + "}\n"); return 0; }
        string stoppedPid = scenario == "wrong-stopped" ? "999" : pid;
        string reason = scenario == "missing-reason" ? "" : ",\"reason\":\"" + (scenario == "deadline" ? "deadline" : "operator") + "\"";
        string hooks = scenario == "hooks" ? "1" : "0";
        string counters = scenario == "missing-counters" ? "" : ",\"dropped\":0,\"errors\":0,\"outstanding_calls\":" + (scenario == "outstanding" ? "1" : "0");
        Write(stdout, "{\"kind\":\"stopped\",\"pid\":" + stoppedPid + reason + ",\"hook_count\":" + hooks + counters + "}\n");
        if (Console.ReadLine() != null) return 4;
        if (scenario == "exit-wait") new System.Threading.ManualResetEvent(false).WaitOne();
        return scenario == "exit-failure" ? 5 : 0;
    }

    private static void Write(Stream stream, string value)
    {
        byte[] bytes = Encoding.UTF8.GetBytes(value);
        stream.Write(bytes, 0, bytes.Length);
        stream.Flush();
    }
}
