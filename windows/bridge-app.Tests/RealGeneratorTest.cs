using System.Globalization;
using System.Security.Cryptography;
using System.Text.Json;
using System.Xml.Linq;
using S4Bridge.Core;

namespace S4Bridge.Tests;

internal static class RealGeneratorTest
{
    public static async Task<int> Run(string[] args)
    {
        var resources = Path.Combine(AppContext.BaseDirectory, "Resources");
        var (engine, upstream, root) = args.Length switch
        {
            1 => (Path.Combine(resources, "s4mk3-hid-midi-bridge.exe"),
                  Path.Combine(resources, "S4 MK3 Bridge Windows.djayMidiMapping"),
                  Path.GetFullPath(args[0])),
            3 => (Path.GetFullPath(args[0]), Path.GetFullPath(args[1]), Path.GetFullPath(args[2])),
            _ => throw new ArgumentException("Expected <fresh-evidence-dir> or <engine> <upstream-mapping> <fresh-evidence-dir>.")
        };
        if (Directory.Exists(root)) throw new IOException("Evidence directory already exists.");
        Directory.CreateDirectory(root);
        var store = new PreferenceStore(resources, Path.Combine(root, "preferences"));
        store.Save();
        var exporter = new MappingExporter(engine, Path.Combine(root, "Temp"));
        var output = new List<string>();
        exporter.Output += line => { lock (output) output.Add(line); };
        var baseline = Path.Combine(root, "baseline.djayMidiMapping");
        await exporter.ExportAsync(baseline, store.Editor.Jog);
        Check.Equal(Hash(upstream), Hash(baseline));
        store.Editor.Jog.ScratchSpeed = 4.1;
        store.Editor.Jog.PitchBendReaction = 42;
        var custom = Path.Combine(root, "unsaved-jog.djayMidiMapping");
        await exporter.ExportAsync(custom, store.Editor.Jog);
        var doc = XDocument.Load(custom);
        foreach (var deck in Enumerable.Range(1, 4))
        {
            var scratch = doc.Descendants("dict").Single(node =>
                Value(node, "keyPath") == $"turntable{deck}.scratchingMove" && Value(node, "midiData") == "5");
            var bend = doc.Descendants("dict").Single(node =>
                Value(node, "keyPath") == $"turntable{deck}.pitchBendMove");
            Check.Equal(4.1, double.Parse(Value(scratch, "rotarySensitivity")!, CultureInfo.InvariantCulture));
            Check.Equal("42", Value(bend, "rotaryAcceleration"));
        }
        Check.Equal(2.7, store.Saved.Jog.ScratchSpeed);
        Check.Equal(17, store.Saved.Jog.PitchBendReaction);
        await Check.RejectAsync(() => exporter.ExportAsync(upstream, store.Editor.Jog));
        var receipt = new
        {
            utc = DateTimeOffset.UtcNow,
            executable = engine,
            executable_sha256 = Hash(engine),
            upstream_mapping_sha256 = Hash(upstream),
            baseline_sha256 = Hash(baseline),
            custom_sha256 = Hash(custom),
            actual_generator_runs = 2,
            scratch_speed_all_four_decks = 4.1,
            bend_reaction_all_four_decks = 42,
            unsaved_editor_proven = true,
            existing_destination_preserved = true,
            native_windows_execution = OperatingSystem.IsWindows(),
            hardware_opened = false
        };
        File.WriteAllText(Path.Combine(root, "generator-proof.json"),
            JsonSerializer.Serialize(receipt, new JsonSerializerOptions { WriteIndented = true }));
        lock (output) File.WriteAllLines(Path.Combine(root, "generator-output.log"), output);
        Console.WriteLine("PASS actual Rust offline generator: exact Windows baseline and unsaved Jog edits on all four decks; no hardware/MIDI access.");
        return 0;
    }
    private static string? Value(XElement node, string key) =>
        node.Elements("key").SingleOrDefault(e => e.Value == key)?.ElementsAfterSelf().First().Value;
    private static string Hash(string path)
    {
        using var stream = File.OpenRead(path);
        return Convert.ToHexStringLower(SHA256.HashData(stream));
    }
}
