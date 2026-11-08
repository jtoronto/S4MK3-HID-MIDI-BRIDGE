using System.Text;
using S4Bridge.Models;

namespace S4Bridge.Core;

public sealed class PreferenceStore
{
    public string Root { get; }
    public Preferences Defaults { get; }
    public Preferences Editor { get; private set; }
    public Preferences Saved { get; private set; }
    public HashSet<string> InvalidFiles { get; } = [];
    public string LoadError { get; private set; } = "";
    public bool RequiresReset => InvalidFiles.Count > 0;
    private readonly string ledDefaults;
    private readonly string jogDefaults;
    private readonly string tensionDefaults;

    public static string LocalRoot()
    {
        var local = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        if (string.IsNullOrWhiteSpace(local))
            throw new IOException("LOCALAPPDATA could not be resolved. No portable-directory fallback is allowed.");
        return Path.Combine(local, "S4 MK3 Bridge");
    }
    public PreferenceStore(string resources, string root)
    {
        Root = Path.GetFullPath(root);
        ledDefaults = File.ReadAllText(Path.Combine(resources, "default-led-config.json"));
        jogDefaults = File.ReadAllText(Path.Combine(resources, "default-jog-config.json"));
        tensionDefaults = File.ReadAllText(Path.Combine(resources, "default-tension-config.json"));
        Defaults = new(new(), PreferenceCodec.Decode<LedPreferences>(ledDefaults, ledDefaults),
            PreferenceCodec.Decode<JogPreferences>(jogDefaults, jogDefaults),
            PreferenceCodec.Decode<TensionPreferences>(tensionDefaults, tensionDefaults), new());
        var d = Defaults.Copy();
        Editor = new(Load("run-settings.json", d.Run), Load("led-config.json", d.Led),
            Load("jog-config.json", d.Jog), Load("tension-config.json", d.Tension),
            Load("screen-settings.json", d.Screen));
        if (InvalidFiles.Contains("screen-settings.json"))
        {
            // Corrupt settings must not erase an otherwise readable recovery intent.
            try
            {
                using var doc = System.Text.Json.JsonDocument.Parse(File.ReadAllText(Path.Combine(Root, "screen-settings.json")));
                if (doc.RootElement.TryGetProperty("restorePending", out var pending)
                   && pending.ValueKind == System.Text.Json.JsonValueKind.True)
                    Editor.Screen.RestorePending = true;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or System.Text.Json.JsonException or InvalidOperationException)
            { LoadError += "Recovery intent could not be fully read: " + ex.Message + "\n"; }
        }
        Saved = Editor.Copy();
    }
    private T Load<T>(string name, T defaults) where T : class
    {
        var path = Path.Combine(Root, name);
        try
        {
            return PreferenceCodec.Decode<T>(File.ReadAllText(path), PreferenceCodec.Encode(defaults));
        }
        catch (FileNotFoundException) { return defaults; }
        catch (DirectoryNotFoundException) { return defaults; }
        catch (Exception ex) when (ex is IOException or InvalidDataException or UnauthorizedAccessException
                                  or System.Text.Json.JsonException or InvalidOperationException)
        {
            InvalidFiles.Add(name);
            LoadError += $"{name}: {ex.Message}\n";
            return defaults;
        }
    }
    public void Save()
    {
        if (RequiresReset) throw new InvalidDataException("Invalid saved files preserved. Reset or explicitly recover them before saving.");
        var snapshot = Editor.Copy();
        var data = new Dictionary<string, string>
        {
            ["run-settings.json"] = PreferenceCodec.Encode(snapshot.Run),
            ["led-config.json"] = PreferenceCodec.Encode(snapshot.Led),
            ["jog-config.json"] = PreferenceCodec.Encode(snapshot.Jog),
            ["tension-config.json"] = PreferenceCodec.Encode(snapshot.Tension),
            ["screen-settings.json"] = PreferenceCodec.Encode(snapshot.Screen)
        };
        // All validation/encoding precedes all writes. Per-file atomicity only.
        foreach (var (name, json) in data)
        {
            try { AtomicWrite(Path.Combine(Root, name), json); }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                throw new IOException($"Save failed at {name}; earlier files may already be saved. Bridge was not restarted.", ex);
            }
            Saved = name switch
            {
                "run-settings.json" => Saved with { Run = snapshot.Run },
                "led-config.json" => Saved with { Led = snapshot.Led },
                "jog-config.json" => Saved with { Jog = snapshot.Jog },
                "tension-config.json" => Saved with { Tension = snapshot.Tension },
                "screen-settings.json" => Saved with { Screen = snapshot.Screen },
                _ => throw new InvalidOperationException("Unknown internal preference file.")
            };
        }
    }
    public void SaveScreen(ScreenPreferences screen)
    {
        if (InvalidFiles.Contains("screen-settings.json"))
            throw new InvalidDataException("Reset invalid screen preferences before saving.");
        AtomicWrite(Path.Combine(Root, "screen-settings.json"), PreferenceCodec.Encode(screen));
        Editor = Editor with { Screen = screen with { } };
        Saved = Saved with { Screen = screen with { } };
    }
    public void Reset()
    {
        var pending = Editor.Screen.RestorePending || Saved.Screen.RestorePending;
        Editor = Defaults.Copy();
        Editor.Screen.RestorePending = pending;
        InvalidFiles.Clear();
        LoadError = "";
    }
    public void ResetMotorized()
    {
        var target = Editor.Jog;
        var defaults = Defaults.Jog;
        foreach (var property in typeof(JogPreferences).GetProperties().Where(p => p.CanWrite &&
                     (p.Name.StartsWith("Platter", StringComparison.Ordinal) ||
                      p.Name.StartsWith("Tt", StringComparison.Ordinal) || p.Name == "VinylEnabledAtStartup")))
            property.SetValue(target, property.GetValue(defaults));
    }
    public void ImportLed(string json)
    {
        var value = PreferenceCodec.Decode<LedPreferences>(json, ledDefaults);
        Editor = Editor with { Led = value };
        // An LED import cannot conceal corruption in another saved file.
        InvalidFiles.Remove("led-config.json");
    }
    public string ExportLed() => PreferenceCodec.Encode(Editor.Led);
    public static void AtomicWrite(string path, string text)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(path))!);
        var temp = path + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try
        {
            using (var file = new FileStream(temp, FileMode.CreateNew, FileAccess.Write, FileShare.None,
                       4096, FileOptions.WriteThrough))
            {
                file.Write(Encoding.UTF8.GetBytes(text));
                file.Flush(true);
            }
            File.Move(temp, path, true);
        }
        finally { if (File.Exists(temp)) File.Delete(temp); }
    }
}
