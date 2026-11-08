using System.Buffers.Binary;
using System.IO.Compression;
using System.Security.Cryptography;

namespace S4Bridge.Tests;

internal static class PackagingTests
{
    private static byte[] PeImage(ushort machine)
    {
        var bytes = new byte[70];
        BinaryPrimitives.WriteUInt16LittleEndian(bytes, 0x5a4d);
        BinaryPrimitives.WriteInt32LittleEndian(bytes.AsSpan(0x3c), 64);
        BinaryPrimitives.WriteUInt32LittleEndian(bytes.AsSpan(64), 0x4550);
        BinaryPrimitives.WriteUInt16LittleEndian(bytes.AsSpan(68), machine);
        return bytes;
    }

    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "packaging accepts AMD64 PE executables", () =>
        {
            using var fixture = new Fixture();
            var path = fixture.PathFor("helper.exe");
            File.WriteAllBytes(path, PeImage(0x8664));
            PackageValidation.RequireX64Executable(path);
        });
        Check.Add(tests, "packaging rejects non-AMD64 helper executables", () =>
        {
            using var fixture = new Fixture();
            var path = fixture.PathFor("helper.exe");
            File.WriteAllBytes(path, PeImage(0x014c));
            Check.Reject(() => PackageValidation.RequireX64Executable(path));
        });
        Check.Add(tests, "packaging rejects truncated PE headers", () =>
        {
            using var fixture = new Fixture();
            var path = fixture.PathFor("helper.exe");
            File.WriteAllBytes(path, new byte[2]);
            Check.Reject(() => PackageValidation.RequireX64Executable(path));
        });
        Check.Add(tests, "packaging verifies path-source Cargo manifest provenance", () =>
        {
            using var fixture = new Fixture();
            var path = fixture.PathFor("source.zip");
            var content = System.Text.Encoding.UTF8.GetBytes("[package]\nname = \"encdr\"\n");
            using (var archive = ZipFile.Open(path, ZipArchiveMode.Create))
            using (var entry = archive.CreateEntry("encdr/Cargo.toml").Open())
                entry.Write(content);
            var hash = Convert.ToHexStringLower(SHA256.HashData(content));
            PackageValidation.RequireArchivedManifest(path, "encdr/Cargo.toml", hash);
            Check.Reject(() => PackageValidation.RequireArchivedManifest(path, "encdr/Cargo.toml", "wrong"));
        });
        Check.Add(tests, "integrated packaging requires reviewed process-wrap inventory", () =>
        {
            using var fixture = new Fixture();
            var root = fixture.PathFor("inventory");
            Directory.CreateDirectory(root);
            File.WriteAllText(Path.Combine(root, "rust-license-manifest.json"), """{"packages":[]}""");
            Check.Reject(() => PackageValidation.RequireReviewedRustInventory(root, "process-wrap"));
        });
    }
}
