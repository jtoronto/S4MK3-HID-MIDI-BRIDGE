using System.Text.Json;

namespace S4Bridge.Platform;

// Inventory is not authority to stop a service or evidence of display access.
public sealed record ScreenAccess(bool Available, bool? DriverRunning, string Detail);
public interface INIServiceControl
{
    ScreenAccess Inspect();
}

public sealed class NIServiceControl(string resources) : INIServiceControl
{
    public ScreenAccess Inspect()
    {
        var path = Path.Combine(resources, "screen-access.json");
        if (File.Exists(path))
        {
            using var doc = JsonDocument.Parse(File.ReadAllText(path));
            var root = doc.RootElement;
            if (root.GetProperty("version").GetInt32() != 1
               || root.GetProperty("route").GetString() != "engine_managed_ni_session"
               || !root.GetProperty("accepted").GetBoolean()
               || string.IsNullOrWhiteSpace(root.GetProperty("evidence").GetString()))
                throw new InvalidDataException("Screen-access declaration is not an accepted engine-managed NI session.");
            return new(true, null,
                "Engine-managed NI session selected by the package acceptance declaration. " +
                "Current status follows the screen worker, not NI service state. Physical acceptance remains separate.");
        }
        return new(false, null,
            "Screen access currently unavailable: no accepted engine-managed access declaration is packaged. " +
            "MI_04 uses the NI niks4m3bd driver. A service stop does not establish WinUSB access. " +
            "Controls, MIDI, LEDs and jog can run without screens. No service or driver changes are performed.");
    }
}
