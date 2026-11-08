using Avalonia;
using Avalonia.Controls;
using Avalonia.Platform.Storage;
using S4Bridge.Core;
using S4Bridge.Platform;

namespace S4Bridge.Views;

internal sealed partial class SettingsView
{
    private static readonly FilePickerFileType JsonType = new("JSON") { Patterns = ["*.json"] };
    private static readonly FilePickerFileType MappingType = new("Djay MIDI mapping") { Patterns = ["*.djayMidiMapping"] };
    private Avalonia.Platform.Storage.IStorageProvider Storage =>
        TopLevel.GetTopLevel(this)?.StorageProvider
            ?? throw new IOException("File dialogs require the attached settings window.");
    private async Task ExportMappingAsync(bool install)
    {
        if (!fields.Valid) throw new InvalidDataException(fields.Error);
        if (host.Exporter.IsExporting) throw new InvalidOperationException("Mapping export already in progress.");
        if (install)
        {
            var staged = await host.Exporter.StageInstallAsync(host.Store.Root, host.Store.Editor.Jog);
            try { await OpenPath(staged); }
            catch (Exception ex) { throw new IOException($"Mapping retained at {staged}, but Windows could not open it in Djay.", ex); }
            await NoticeAsync("Mapping ready for import", $"A generated mapping is retained at {staged}.\n\nComplete the import in Djay, then select it for LoopBe Internal MIDI. Opening the file does not confirm import; existing mappings were not overwritten.");
            return;
        }
        var music = Environment.GetFolderPath(Environment.SpecialFolder.MyMusic);
        var mappings = Path.Combine(music, "djay", "MIDI Mappings");
        var folder = Directory.Exists(mappings)
            ? await Storage.TryGetFolderFromPathAsync(new Uri(mappings + Path.DirectorySeparatorChar))
            : null;
        var file = await Storage.SaveFilePickerAsync(new()
        {
            Title = "Export Djay mapping",
            SuggestedFileName = "S4 MK3 Bridge Windows.djayMidiMapping",
            SuggestedStartLocation = folder,
            DefaultExtension = "djayMidiMapping",
            FileTypeChoices = [MappingType],
            ShowOverwritePrompt = true
        });
        if (file is null) return;
        var destination = file.TryGetLocalPath() ?? throw new IOException("Select a local mapping destination.");
        await host.Exporter.ExportAsync(destination, host.Store.Editor.Jog);
        await NoticeAsync("Mapping saved", "Select the new Windows mapping for LoopBe Internal MIDI in Djay. Existing user mappings were preserved.");
    }
    private async Task ImportLedAsync()
    {
        var files = await Storage.OpenFilePickerAsync(new()
        {
            Title = "Import LED preferences",
            AllowMultiple = false,
            FileTypeFilter = [JsonType]
        });
        if (files.Count == 0) return;
        await using var stream = await files[0].OpenReadAsync();
        if (stream.CanSeek && stream.Length > 1024 * 1024) throw new IOException("LED import exceeds 1 MiB.");
        using var reader = new StreamReader(stream);
        host.Store.ImportLed(await reader.ReadToEndAsync());
        host.Edited(); BuildTabs();
    }
    private async Task ExportLedAsync()
    {
        if (!fields.Valid) throw new InvalidDataException(fields.Error);
        var json = host.Store.ExportLed();
        var file = await Storage.SaveFilePickerAsync(new()
        {
            Title = "Export LED and Palette preferences",
            SuggestedFileName = "s4-led-config.json",
            DefaultExtension = "json",
            FileTypeChoices = [JsonType],
            ShowOverwritePrompt = true
        });
        if (file is null) return;
        var path = file.TryGetLocalPath() ?? throw new IOException("Select a local JSON file.");
        PreferenceStore.AtomicWrite(path, json);
    }
}
