using Avalonia.Controls;
using Avalonia.LogicalTree;
using S4Bridge.Core;
using S4Bridge.Views;

namespace S4Bridge.Tests;

internal static class BloopDependencyTests
{
    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "BLOOP installer action clears after a successful recheck", () =>
        {
            using var fixture = new Fixture();
            var results = new Queue<BloopDependencyProbeResult>([
                new(BloopDependencyStatus.TransportUnavailable),
                new(BloopDependencyStatus.Available)
            ]);
            var host = new HostController(fixture.Resources, fixture.Root, checkBloop: results.Dequeue);
            using var diagnostics = host.Diagnostics;
            var view = new SettingsView(host);
            try
            {
                var bridge = view.BuiltTabs[0];
                var install = bridge.GetLogicalDescendants().OfType<Button>()
                    .Single(button => Equals(button.Content, "Download from Microsoft"));
                Check.True(!install.IsVisible);

                host.RefreshBloopAsync().WaitAsync(TimeSpan.FromSeconds(5)).GetAwaiter().GetResult();
                view.RefreshState();
                Check.True(install.IsVisible);

                host.RefreshBloopAsync().WaitAsync(TimeSpan.FromSeconds(5)).GetAwaiter().GetResult();
                view.RefreshState();
                Check.True(!install.IsVisible);
                Check.True(!host.Supervisor.HasOwnedResources);
            }
            finally { view.StopRefresh(); }
        });
    }
}
