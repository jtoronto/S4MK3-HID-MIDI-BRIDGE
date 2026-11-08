using System.IO.Compression;
using System.Security.Cryptography;
using System.Text.Json;
using System.Xml.Linq;

if (args.Length != 7) throw new ArgumentException("Expected publish-root, assets-json, retained-inventory-root, engine-mode, screen-supervisor, screen-worker, screen-inventory.");
var app = Path.GetFullPath(args[0]);
var inventory = args[2].Length > 0 ? Path.GetFullPath(args[2]) : null;
var hostOnly = args[3] == "host-only";
var screenAccess = Path.Combine(app, "Resources", "screen-access.json");
var screenEnabled = File.Exists(screenAccess);
var options = new JsonSerializerOptions { WriteIndented = true };
static string Hash(string path)
{
    using var stream = File.OpenRead(path);
    return Convert.ToHexStringLower(SHA256.HashData(stream));
}
static void CopyTree(string source, string destination)
{
    Directory.CreateDirectory(destination);
    foreach (var path in Directory.EnumerateFiles(source, "*", SearchOption.AllDirectories))
    {
        var target = Path.Combine(destination, Path.GetRelativePath(source, path));
        Directory.CreateDirectory(Path.GetDirectoryName(target)!); File.Copy(path, target, false);
    }
}
PackageValidation.RequireX64Executable(Path.Combine(app, "S4Bridge.exe"));
var engine = Path.Combine(app, "Resources", "s4mk3-hid-midi-bridge.exe");
var screenSupervisor = args[4];
var screenWorker = args[5];
var screenInventory = args[6];
var screenInventoryIncluded = screenEnabled && screenInventory.Length > 0;
if (!hostOnly)
{
    PackageValidation.RequireX64Executable(engine);
    var mapping = XDocument.Load(Path.Combine(app, "Resources", "S4 MK3 Bridge Windows.djayMidiMapping"));
    if (mapping.Root?.Name.LocalName != "plist" ||
        !mapping.Descendants("string").Any(e => e.Value == "LoopBe Internal MIDI"))
        throw new InvalidDataException("Bundled mapping lacks Windows LoopBe identity.");
}
else if (File.Exists(engine)) throw new InvalidDataException("Host-only publish must not contain an engine.");
if (screenEnabled)
{
    if (hostOnly || string.IsNullOrWhiteSpace(screenSupervisor) ||
        string.IsNullOrWhiteSpace(screenWorker))
        throw new InvalidDataException("Screen access requires integrated mode and both helpers.");
    var supervisor = Path.GetFullPath(screenSupervisor);
    var worker = Path.GetFullPath(screenWorker);
    var supervisorTarget = Path.Combine(app, "Resources", "s4-screen-supervisor.exe");
    var workerTarget = Path.Combine(app, "Resources", "s4-encdr-screen-worker.exe");
    PackageValidation.RequireX64Executable(supervisor);
    PackageValidation.RequireX64Executable(worker);
    File.Copy(supervisor, supervisorTarget, false);
    File.Copy(worker, workerTarget, false);

    if (screenInventoryIncluded)
    {
        var helperInventory = Path.GetFullPath(screenInventory);
        var helperManifestPath = Path.Combine(helperInventory, "screen-helper-manifest.json");
        if (!Directory.Exists(Path.Combine(helperInventory, "notices")) ||
            !File.Exists(Path.Combine(helperInventory, "rust-license-manifest.json")) ||
            !File.Exists(helperManifestPath))
            throw new InvalidDataException("Screen inventory must contain screen-helper-manifest.json, rust-license-manifest.json, and notices/.");
        using var helperManifest = JsonDocument.Parse(File.ReadAllText(helperManifestPath));
        var manifest = helperManifest.RootElement;
        if (manifest.GetProperty("schema_version").GetInt32() != 1 ||
            manifest.GetProperty("supervisor_sha256").GetString() != Hash(supervisor) ||
            manifest.GetProperty("worker_sha256").GetString() != Hash(worker))
            throw new InvalidDataException("Screen inventory does not identify these exact helper executables.");
        var sourceArchive = manifest.GetProperty("source_archive").GetString();
        if (string.IsNullOrWhiteSpace(sourceArchive) || Path.IsPathRooted(sourceArchive))
            throw new InvalidDataException("Screen inventory must identify its corresponding source ZIP archive.");
        var sourcePath = Path.GetFullPath(Path.Combine(helperInventory, sourceArchive));
        if (!sourcePath.StartsWith(helperInventory + Path.DirectorySeparatorChar, StringComparison.Ordinal) ||
            !File.Exists(sourcePath) || Hash(sourcePath) != manifest.GetProperty("source_sha256").GetString())
            throw new InvalidDataException("Screen helper source archive is missing or changed.");
        var encdrManifestPath = manifest.GetProperty("encdr_manifest_path").GetString();
        var encdrManifestHash = manifest.GetProperty("encdr_manifest_sha256").GetString();
        if (string.IsNullOrWhiteSpace(encdrManifestPath) || string.IsNullOrWhiteSpace(encdrManifestHash) ||
            Path.IsPathRooted(encdrManifestPath) ||
            encdrManifestPath.Contains('\\') || encdrManifestPath.Split('/').Any(part => part is "" or "." or ".."))
            throw new InvalidDataException("Screen inventory must identify Encdr's Cargo.toml inside the source ZIP.");
        PackageValidation.RequireArchivedManifest(sourcePath, encdrManifestPath, encdrManifestHash);

        using var screenLicenses = JsonDocument.Parse(File.ReadAllText(
            Path.Combine(helperInventory, "rust-license-manifest.json")));
        if (manifest.GetProperty("rust_license_manifest_sha256").GetString() !=
            Hash(Path.Combine(helperInventory, "rust-license-manifest.json")))
            throw new InvalidDataException("Screen helper license manifest does not match its reviewed inventory.");
        var helperPackages = screenLicenses.RootElement.GetProperty("packages").EnumerateArray().ToArray();
        var encdrPackage = helperPackages.SingleOrDefault(package =>
            package.GetProperty("name").GetString() == "encdr");
        const string encdrGitSource = "git+https://github.com/RufusIbiza/Encdr.git?" +
            "rev=a5414658a69c4330d10b95f2359371dfc8550326#" +
            "a5414658a69c4330d10b95f2359371dfc8550326";
        if (encdrPackage.ValueKind == JsonValueKind.Undefined ||
            (encdrPackage.GetProperty("source").ValueKind != JsonValueKind.Null &&
             (encdrPackage.GetProperty("source").ValueKind != JsonValueKind.String ||
              encdrPackage.GetProperty("source").GetString() != encdrGitSource)) ||
            encdrPackage.GetProperty("manifest_sha256").GetString() != encdrManifestHash ||
            encdrPackage.GetProperty("license").GetString()?.Contains("GPL-3.0", StringComparison.Ordinal) != true)
            throw new InvalidDataException("Screen helper inventory must verify the GPL Encdr source by Cargo.toml SHA-256.");
        PackageValidation.RequireReviewedRustInventory(helperInventory, "encdr");
        if (encdrPackage.GetProperty("notice_files").GetArrayLength() == 0)
            throw new InvalidDataException("Screen helper license inventory does not include the pinned Encdr notice.");
        CopyTree(Path.Combine(helperInventory, "notices"), Path.Combine(app, "screen-notices"));
        Directory.CreateDirectory(Path.Combine(app, "sources"));
        File.Copy(sourcePath, Path.Combine(app, "sources", "screen-helper-source.zip"), false);
        File.Copy(helperManifestPath, Path.Combine(app, "screen-helper-manifest.json"), false);
        File.Copy(Path.Combine(helperInventory, "rust-license-manifest.json"),
            Path.Combine(app, "screen-rust-license-manifest.json"), false);
    }
}
else if (!string.IsNullOrWhiteSpace(screenSupervisor) || !string.IsNullOrWhiteSpace(screenWorker) ||
    screenInventory.Length > 0)
    throw new InvalidDataException("Screen helper inputs require screen-access.json.");
using var assets = JsonDocument.Parse(File.ReadAllText(args[1]));
var packagesRoot = assets.RootElement.GetProperty("packageFolders").EnumerateObject().First().Name;
if (inventory is not null)
{
    if (!hostOnly) PackageValidation.RequireReviewedRustInventory(inventory, "process-wrap");
    using var approved = JsonDocument.Parse(File.ReadAllText(Path.Combine(inventory, "nuget-license-manifest.json")));
    var licenses = new List<LicenseEntry>();
    foreach (var lib in assets.RootElement.GetProperty("libraries").EnumerateObject())
    {
        if (lib.Value.GetProperty("type").GetString() != "package") continue;
        var pair = lib.Name.Split('/');
        var name = pair[0]; var version = pair[1];
        var cached = Path.Combine(packagesRoot, lib.Value.GetProperty("path").GetString()!);
        var package = Directory.GetFiles(cached, "*.nupkg").Single();
        var packageHash = Hash(package);
        var entry = approved.RootElement.GetProperty("packages").EnumerateArray().SingleOrDefault(p =>
            p.GetProperty("name").GetString() == name && p.GetProperty("version").GetString() == version);
        if (entry.ValueKind == JsonValueKind.Undefined || entry.GetProperty("package_sha256").GetString() != packageHash)
            throw new InvalidDataException($"Package not in retained reviewed inventory: {name}/{version}");
        licenses.Add(new(name, version, packageHash, entry.GetProperty("license").GetString()!));
    }
    CopyTree(Path.Combine(inventory, "notices"), Path.Combine(app, "notices"));
    File.WriteAllText(Path.Combine(app, "nuget-license-manifest.json"), JsonSerializer.Serialize(licenses, options));
    if (!hostOnly)
        File.Copy(Path.Combine(inventory, "rust-license-manifest.json"), Path.Combine(app, "rust-license-manifest.json"), false);
}
// These notices come from the exact retained runtime pack, never a machine runtime.
var runtime = Path.Combine(packagesRoot, "microsoft.netcore.app.runtime.win-x64", "10.0.14");
Directory.CreateDirectory(Path.Combine(app, "notices", "dotnet-10.0.14"));
foreach (var file in new[] { "LICENSE.TXT", "THIRD-PARTY-NOTICES.TXT" })
    File.Copy(Path.Combine(runtime, file), Path.Combine(app, "notices", "dotnet-10.0.14", file), false);
var description = hostOnly
    ? "HOST-ONLY ENGINE-INTEGRATION HANDOFF. No Rust executable is bundled. Start, Check and mapping export remain unavailable. This is not a complete Windows bridge product."
    : "INTEGRATION PACKAGE, NOT PHYSICALLY ACCEPTED. Actual Rust binary included. Verify engine identity, Windows GUI, MIDI, HID, motors, screens and audio before product acceptance.";
File.WriteAllText(Path.Combine(app, "PACKAGE-STATUS.txt"), description +
    "\nSelf-contained .NET 10.0.14 win-x64. Run S4Bridge.exe. No Rust/Cargo/.NET tools required." +
    "\nScreen route is unavailable unless an accepted engine-managed declaration is supplied; no automatic NI stop." +
    "\nOriginal Djay mappings are create-only preserved. Host preferences: LOCALAPPDATA\\S4 MK3 Bridge." +
    (inventory is null
        ? "\nEngine and host dependency inventory is not included; review distribution requirements before sharing."
        : "") +
    (screenEnabled && !screenInventoryIncluded
        ? "\nScreen helper source and notices are not included; review distribution requirements before sharing."
        : "") +
    "\nThis archive is unsigned; no driver/installer is bundled.\n");
var files = Directory.EnumerateFiles(app, "*", SearchOption.AllDirectories)
    .Order(StringComparer.Ordinal).Select(p => new FileEntry(Path.GetRelativePath(app, p).Replace('\\', '/'),
        new FileInfo(p).Length, Hash(p))).ToArray();
File.WriteAllText(Path.Combine(app, "runtime-manifest.json"), JsonSerializer.Serialize(new
{
    version = 1,
    created_utc = DateTimeOffset.UtcNow,
    kind = hostOnly ? "host-only" : "engine-integration",
    runtime = "10.0.14",
    rid = "win-x64",
    host_only = hostOnly,
    dependency_inventory_included = inventory is not null,
    screen_access_enabled = screenEnabled,
    screen_inventory_included = screenInventoryIncluded,
    files
}, options));
var zip = app + ".zip";
if (File.Exists(zip)) throw new IOException("Archive already exists: " + zip);
ZipFile.CreateFromDirectory(app, zip, CompressionLevel.Optimal, false);
using (var archive = ZipFile.OpenRead(zip))
{
    foreach (var entry in archive.Entries.Where(e => e.Name.Length > 0))
    {
        using var input = entry.Open();
        var actual = Convert.ToHexStringLower(SHA256.HashData(input));
        if (actual != Hash(Path.Combine(app, entry.FullName)))
            throw new InvalidDataException("Archive hash mismatch: " + entry.FullName);
    }
}
File.WriteAllText(zip + ".sha256", Hash(zip) + "  " + Path.GetFileName(zip) + "\n");
Console.WriteLine($"PACKAGED {files.Length + 1} files; {zip}; sha256={Hash(zip)}");

internal sealed record LicenseEntry(string Name, string Version, string PackageSha256, string License);
internal sealed record FileEntry(string Path, long Bytes, string Sha256);
