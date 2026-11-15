using System.Diagnostics;
using System.Xml.Linq;
using S4Bridge.Models;

namespace S4Bridge.Core;

public sealed class MappingExporter(string executable, string temporaryRoot)
{
    private int exporting;
    public bool IsExporting => Volatile.Read(ref exporting) != 0;
    public event Action<string>? Output;
    public async Task<string> StageInstallAsync(string root, JogPreferences editor)
    {
        var destination = Path.Combine(root, "Generated Mappings", Guid.NewGuid().ToString("N"),
            "S4 MK3 Bridge Windows.djayMidiMapping");
        await ExportAsync(destination, editor);
        return destination;
    }
    public async Task ExportAsync(string destination, JogPreferences editor)
    {
        if (Interlocked.Exchange(ref exporting, 1) != 0)
            throw new InvalidOperationException("Mapping export already in progress.");
        string? temporary = null;
        try
        {
            // Snapshot and validate before launching the actual packaged generator.
            var config = PreferenceCodec.Encode(editor);
            if (File.Exists(destination) || Directory.Exists(destination))
                throw new IOException("Choose a new mapping filename. Existing user mappings are preserved.");
            temporary = Path.Combine(temporaryRoot, "S4BridgeMapping-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(temporary);
            var jog = Path.Combine(temporary, "jog-config.json");
            var output = Path.Combine(temporary, "S4 MK3 Bridge Windows.djayMidiMapping");
            PreferenceStore.AtomicWrite(jog, config);
            var info = new ProcessStartInfo(executable)
            {
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                RedirectStandardInput = true
            };
            foreach (var arg in new[] { "--generate-mapping", output, "--midi-backend", BloopEndpoint.Backend, "--jog-config", jog })
                info.ArgumentList.Add(arg);
            using var child = new Process { StartInfo = info };
            child.Start();
            child.StandardInput.Close();
            var stdout = ReadOutput(child.StandardOutput);
            var stderr = ReadOutput(child.StandardError);
            try { await child.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(30)); }
            catch (TimeoutException)
            {
                // Offline generation owns no HID/MIDI. Reap only this owned utility.
                child.Kill(true);
                await child.WaitForExitAsync();
                await Task.WhenAll(stdout, stderr);
                throw new IOException("Offline mapping generator timed out.");
            }
            await Task.WhenAll(stdout, stderr);
            Output?.Invoke($"EXPORT_EXIT: {child.ExitCode}");
            if (child.ExitCode != 0) throw new IOException($"Mapping generator failed with status {child.ExitCode}.");
            if (!File.Exists(output)) throw new IOException("Mapping generator produced no file.");
            var doc = XDocument.Load(output);
            var endpoint = doc.Root?.Element("dict")?.Elements("key")
                .SingleOrDefault(key => key.Value == "endpointName")?.ElementsAfterSelf().FirstOrDefault();
            if (doc.Root?.Name.LocalName != "plist"
                || endpoint?.Name.LocalName != "string" || endpoint.Value != BloopEndpoint.PortName)
                throw new InvalidDataException("Generated mapping lacks the Windows BLOOP endpoint identity.");
            // A sibling staged file permits atomic, create-new installation across volumes.
            var parent = Path.GetDirectoryName(Path.GetFullPath(destination))!;
            Directory.CreateDirectory(parent);
            var staging = Path.Combine(parent, ".s4-mapping-" + Guid.NewGuid().ToString("N"));
            try
            {
                File.Copy(output, staging, false);
                File.Move(staging, destination, false);
            }
            finally { if (File.Exists(staging)) File.Delete(staging); }
        }
        finally
        {
            try { if (temporary is not null) Directory.Delete(temporary, true); }
            finally { Volatile.Write(ref exporting, 0); }
        }
    }
    private async Task ReadOutput(StreamReader reader)
    {
        var buffer = new char[4096];
        while (await reader.ReadAsync(buffer) is var count && count > 0)
            Output?.Invoke(new string(buffer, 0, count));
    }
}
