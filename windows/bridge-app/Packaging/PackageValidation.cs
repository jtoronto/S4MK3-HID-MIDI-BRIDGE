using System.IO.Compression;
using System.Security.Cryptography;
using System.Text.Json;

internal static class PackageValidation
{
    private static string Hash(Stream stream) =>
        Convert.ToHexStringLower(SHA256.HashData(stream));

    public static void RequireX64Executable(string path)
    {
        using var stream = File.OpenRead(path);
        using var reader = new BinaryReader(stream);
        if (reader.ReadUInt16() != 0x5a4d) throw new InvalidDataException($"Not a PE: {path}");
        stream.Position = 0x3c;
        var offset = reader.ReadInt32();
        if (offset < 0 || offset > stream.Length - 6)
            throw new InvalidDataException($"Invalid PE header: {path}");
        stream.Position = offset;
        if (reader.ReadUInt32() != 0x4550 || reader.ReadUInt16() != 0x8664)
            throw new InvalidDataException($"Not AMD64 PE: {path}");
    }

    public static void RequireArchivedManifest(string archivePath, string relativePath, string expectedHash)
    {
        if (Path.IsPathRooted(relativePath) || relativePath.Contains('\\') ||
            relativePath.Split('/').Any(part => part is "" or "." or ".."))
            throw new InvalidDataException("Invalid path-source manifest path.");
        using var archive = ZipFile.OpenRead(archivePath);
        var entry = archive.GetEntry(relativePath)
            ?? throw new InvalidDataException($"Path-source manifest is absent: {relativePath}");
        using var stream = entry.Open();
        if (Hash(stream) != expectedHash)
            throw new InvalidDataException($"Path-source manifest SHA-256 mismatch: {relativePath}");
    }

    public static void RequireReviewedRustInventory(string inventoryRoot, string requiredPackage)
    {
        var root = Path.GetFullPath(inventoryRoot);
        using var manifest = JsonDocument.Parse(File.ReadAllText(Path.Combine(root, "rust-license-manifest.json")));
        var packages = manifest.RootElement.GetProperty("packages").EnumerateArray().ToArray();
        if (!packages.Any(package => package.GetProperty("name").GetString() == requiredPackage))
            throw new InvalidDataException($"Reviewed Rust inventory is missing required package: {requiredPackage}");
        foreach (var package in packages)
        {
            if (string.IsNullOrWhiteSpace(package.GetProperty("license").GetString()))
                throw new InvalidDataException("Reviewed Rust inventory contains a package without license metadata.");
            foreach (var notice in package.GetProperty("notice_files").EnumerateArray())
            {
                var relative = notice.GetString()
                    ?? throw new InvalidDataException("Invalid Rust notice path.");
                if (Path.IsPathRooted(relative) || relative.Contains('\\') ||
                    relative.Split('/').Any(part => part is "" or "." or ".."))
                    throw new InvalidDataException($"Invalid Rust notice path: {relative}");
                var path = Path.GetFullPath(Path.Combine(root, relative.Replace('/', Path.DirectorySeparatorChar)));
                if (!path.StartsWith(root + Path.DirectorySeparatorChar, StringComparison.Ordinal) ||
                    !File.Exists(path))
                    throw new InvalidDataException($"Reviewed Rust notice is missing: {relative}");
                var expectedHash = package.GetProperty("notice_provenance")
                    .GetProperty(relative).GetProperty("sha256").GetString();
                using var stream = File.OpenRead(path);
                if (Hash(stream) != expectedHash)
                    throw new InvalidDataException($"Reviewed Rust notice SHA-256 mismatch: {relative}");
            }
        }
    }
}
