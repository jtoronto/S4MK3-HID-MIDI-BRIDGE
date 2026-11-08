using System.Reflection;

namespace S4Bridge.Tests;

internal static class Program
{
    public static async Task<int> Main(string[] args)
    {
        if (args.Contains("--generate-mapping")) return ChildFixture.Generate(args);
        if (args.FirstOrDefault() == "--fixture") return await ChildFixture.Run(args[1..]);
        if (args.FirstOrDefault() == "--real-generator") return await RealGeneratorTest.Run(args[1..]);
        var tests = new List<(string, Func<Task>)>();
        RenderedLogTests.Register(tests);
        PreferenceTests.Register(tests);
        ProtocolTests.Register(tests);
        MidiCaptureTests.Register(tests);
        DiagnosticsTests.Register(tests);
        ControlTests.Register(tests);
        PackagingTests.Register(tests);
        ProcessTests.Register(tests);
        var failed = 0;
        foreach (var (name, test) in tests)
        {
            try { await test(); Console.WriteLine("PASS " + name); }
            catch (Exception ex) { failed++; Console.Error.WriteLine($"FAIL {name}\n{ex}"); }
        }
        Console.WriteLine($"RESULT {tests.Count - failed}/{tests.Count} passed; {failed} failed");
        return failed == 0 ? 0 : 1;
    }
}

internal static class Check
{
    public static void True(bool value, string detail = "Expected true")
    { if (!value) throw new InvalidOperationException(detail); }
    public static void Equal<T>(T expected, T actual)
    { if (!EqualityComparer<T>.Default.Equals(expected, actual)) throw new InvalidOperationException($"Expected {expected}; actual {actual}"); }
    public static void Reject(Action action)
    {
        try { action(); }
        catch (Exception ex) when (ex is System.Text.Json.JsonException or InvalidDataException
            or IOException or InvalidOperationException or FormatException or ArgumentException)
        { return; }
        throw new InvalidOperationException("Expected rejection, but operation succeeded.");
    }
    public static async Task RejectAsync(Func<Task> action)
    {
        try { await action(); }
        catch (Exception ex) when (ex is IOException or InvalidOperationException or System.Text.Json.JsonException) { return; }
        throw new InvalidOperationException("Expected rejection, but async operation succeeded.");
    }
    public static void Add(List<(string, Func<Task>)> tests, string name, Action action) =>
        tests.Add((name, () => { action(); return Task.CompletedTask; }
    ));
}

internal sealed class Fixture : IDisposable
{
    public string Root { get; }
    public string Resources => Path.Combine(AppContext.BaseDirectory, "Resources");
    public Fixture()
    {
        var root = Environment.GetEnvironmentVariable("S4_TEST_ROOT")
            ?? Path.Combine(AppContext.BaseDirectory, "test-artifacts");
        Root = Path.Combine(root, Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(Root);
    }
    public string PathFor(string file) => Path.Combine(Root, file);
    public void Dispose() => Directory.Delete(Root, true);
    public static (string Exe, string[] Args) Child(params string[] args)
    {
        var exe = Environment.ProcessPath ?? throw new IOException("Process path unavailable.");
        var prefix = Path.GetFileNameWithoutExtension(exe) == "dotnet"
            ? new[] { Assembly.GetExecutingAssembly().Location, "--fixture" } : new[] { "--fixture" };
        return (exe, [.. prefix, .. args]);
    }
}
