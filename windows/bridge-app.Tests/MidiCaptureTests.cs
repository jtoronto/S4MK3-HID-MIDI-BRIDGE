using S4Bridge.Core;
using S4Bridge.Diagnostics;
using System.Text.Json;

namespace S4Bridge.Tests;

internal static class MidiCaptureTests
{
    public static void Register(List<(string, Func<Task>)> tests)
    {
        Check.Add(tests, "capture launch is opt-in, absolute and full-profile only", () =>
        {
            using var fixture = new Fixture();
            var store = new PreferenceStore(fixture.Resources, fixture.Root);
            store.Editor.Run.Profile = "full";
            store.Save();
            var path = fixture.PathFor("capture with spaces.jsonl");
            var ordinary = LaunchSpec.Runtime("engine", store, false);
            Check.True(!ordinary.Arguments.Contains("--midi-capture"));
            var capture = LaunchSpec.Runtime("engine", store, false, path);
            var index = Array.IndexOf(capture.Arguments, "--midi-capture");
            Check.True(index >= 0);
            Check.Equal(path, capture.Arguments[index + 1]);
            Check.Reject(() => LaunchSpec.Runtime("engine", store, false, "relative.jsonl"));
            foreach (var profile in new[] { "minimal", "probe" })
            {
                store.Editor.Run.Profile = profile;
                store.Save();
                Check.Reject(() => LaunchSpec.Runtime("engine", store, false, path));
            }
        });

        Check.Add(tests, "capture creates a new absolute path only when armed for full profile", () =>
        {
            using var fixture = new Fixture();
            var capture = new MidiCapture();
            Check.Equal<string?>(null, capture.PrepareLaunch(fixture.Root, "full"));
            capture.EnabledForNextLaunch = true;
            Check.Reject(() => capture.PrepareLaunch(fixture.Root, "probe"));
            var path = capture.PrepareLaunch(fixture.Root, "full")!;
            Check.True(Path.IsPathFullyQualified(path));
            Check.True(!File.Exists(path));
            Check.True(!capture.EnabledForNextLaunch);
            Check.Equal("pending", capture.State);
            capture.EnabledForNextLaunch = true;
            var next = capture.PrepareLaunch(fixture.Root, "full")!;
            Check.True(path != next && !File.Exists(next));
        });

        Check.Add(tests, "capture ignores stale launches and requires matching started path", () =>
        {
            using var fixture = new Fixture();
            var capture = new MidiCapture { EnabledForNextLaunch = true };
            var path = capture.PrepareLaunch(fixture.Root, "full")!;
            var launch = Guid.NewGuid();
            capture.Launched(launch);
            capture.Accept(Guid.NewGuid(), Status(new { version = 1, state = "started", path, max_file_bytes = 1048576 }));
            Check.Equal("starting", capture.State);
            capture.Accept(launch, Status(new { version = 1, state = "started", path, max_file_bytes = 1048576 }));
            Check.Equal("recording", capture.State);
            capture.Exited(Guid.NewGuid());
            Check.Equal("recording", capture.State);
        });

        Check.Add(tests, "capture rejects started status for another path", () =>
        {
            using var fixture = new Fixture();
            var capture = new MidiCapture { EnabledForNextLaunch = true };
            capture.PrepareLaunch(fixture.Root, "full");
            var launch = Guid.NewGuid();
            capture.Launched(launch);
            capture.Accept(launch, Status(new { version = 1, state = "started",
                path = fixture.PathFor("other"), max_file_bytes = 1048576 }));
            Check.Equal("starting", capture.State);
            Check.True(capture.Error.Length > 0);
            capture.Exited(launch);
            Check.Equal("incomplete", capture.State);
        });

        Check.Add(tests, "marker acknowledgement means queued not persisted", () =>
        {
            using var fixture = new Fixture();
            var capture = Recording(fixture, out var launch, out var path);
            var marker = capture.NextMarker();
            capture.Accept(launch, Status(new { version = 1, state = "marker", request_id = marker.ToString(),
                accepted = true, sequence = 7, elapsed_us = 12, reason = (string?)null }));
            Check.Equal<ulong?>(marker, capture.LastMarkerId);
            Check.Equal<bool?>(true, capture.LastMarkerAccepted);
            Check.Equal<ulong?>(7, capture.LastMarkerSequence);
            Check.Equal("12", capture.LastMarkerElapsedUs);
            Check.Equal(0UL, capture.RecordCount);
            capture.Accept(launch, Completion("finished", path, null));
            Check.Equal("finished", capture.State);
            Check.Equal(8UL, capture.AttemptedCount);
            Check.Equal(5UL, capture.QueuedCount);
            Check.Equal(3UL, capture.RecordCount);
            Check.Equal(3UL, capture.DroppedCount);
            Check.Equal(2UL, capture.UnwrittenCount);
        });

        Check.Add(tests, "rejected and unacknowledged markers remain visible", () =>
        {
            using var fixture = new Fixture();
            var capture = Recording(fixture, out var launch, out var path);
            var marker = capture.NextMarker();
            capture.Accept(launch, Status(new { version = 1, state = "marker", request_id = marker.ToString(),
                accepted = false, sequence = 9, elapsed_us = 13, reason = "queue_full" }));
            Check.Equal<bool?>(false, capture.LastMarkerAccepted);
            Check.True(capture.MarkerError.Contains("queue_full", StringComparison.Ordinal));
            capture.NextMarker();
            capture.Accept(launch, Completion("finished", path, null));
            Check.True(capture.MarkerError.Length > 0);
            Check.Reject(() => capture.NextMarker());
        });

        Check.Add(tests, "failed capture preserves final counts and reports writer failure", () =>
        {
            using var fixture = new Fixture();
            var capture = Recording(fixture, out var launch, out var path);
            capture.Accept(launch, Completion("failed", path, "disk full"));
            Check.Equal("failed", capture.State);
            Check.Equal(3UL, capture.RecordCount);
            Check.Equal(3UL, capture.DroppedCount);
            Check.Equal(2UL, capture.UnwrittenCount);
            Check.True(capture.Error.Contains("disk full", StringComparison.Ordinal));
        });

        Check.Add(tests, "capture limit rejects marker and retains final loss counts", () =>
        {
            using var fixture = new Fixture();
            var capture = Recording(fixture, out var launch, out var path);
            Check.Equal<ulong?>(1048576, capture.MaxFileBytes);
            var queued = capture.NextMarker();
            capture.Accept(launch, Status(new { version = 1, state = "marker",
                request_id = queued.ToString(), accepted = true, sequence = 7,
                elapsed_us = 12, reason = (string?)null }));
            Check.Equal(0UL, capture.RecordCount);
            var rejected = capture.NextMarker();
            capture.Accept(launch, Status(new { version = 1, state = "limited", path,
                reason = "file_limit", max_file_bytes = 1048576 }));
            Check.Equal("limited", capture.State);
            Check.True(capture.Truncated);
            Check.True(!capture.HasFinalCounts);
            Check.Reject(() => capture.NextMarker());
            capture.Accept(launch, Status(new { version = 1, state = "marker",
                request_id = rejected.ToString(), accepted = false, sequence = 8,
                elapsed_us = 13, reason = "recording_limit" }));
            Check.Equal<bool?>(false, capture.LastMarkerAccepted);
            Check.True(capture.MarkerError.Contains("recording_limit", StringComparison.Ordinal));
            Check.Equal(0UL, capture.RecordCount);
            capture.Accept(launch, Status(new { version = 1, state = "finished", path,
                error = (string?)null, counts = new { attempted = 9, queued = 5, written = 3,
                    dropped_full = 1, dropped_oversized = 1, dropped_disconnected = 1,
                    dropped_limit = 2, dropped_limit_queued = 1, unwritten = 2, truncated = true } }));
            Check.Equal("incomplete", capture.State);
            Check.True(capture.Truncated && capture.HasFinalCounts);
            Check.Equal(9UL, capture.AttemptedCount);
            Check.Equal(5UL, capture.QueuedCount);
            Check.Equal(3UL, capture.RecordCount);
            Check.Equal(5UL, capture.DroppedCount);
            Check.Equal(2UL, capture.DroppedLimitCount);
            Check.Equal(2UL, capture.UnwrittenCount);
            Check.Equal(1UL, capture.DroppedLimitQueuedCount);
        });

        Check.Add(tests, "malformed receipt does not hide later final summary", () =>
        {
            using var fixture = new Fixture();
            var capture = Recording(fixture, out var launch, out var path);
            capture.Accept(launch, "S4_MIDI_CAPTURE_STATUS {");
            Check.Equal("recording", capture.State);
            Check.True(capture.Error.Length > 0);
            capture.Accept(launch, Completion("finished", path, null));
            Check.Equal("finished", capture.State);
            Check.Equal(3UL, capture.RecordCount);
            Check.Equal(2UL, capture.UnwrittenCount);
        });

        Check.Add(tests, "exit without final capture status reports incomplete and missing marker", () =>
        {
            using var fixture = new Fixture();
            var capture = Recording(fixture, out var launch, out _);
            capture.NextMarker();
            capture.Exited(launch);
            Check.Equal("incomplete", capture.State);
            Check.True(capture.Error.Length > 0);
            Check.True(capture.MarkerError.Length > 0);
            Check.Reject(() => capture.NextMarker());
        });

        tests.Add(("concurrent marker and screen commands remain complete on supervised pipes", async () =>
        {
            const int count = 32;
            var supervisor = new BridgeSupervisor();
            var ready = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var received = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var start = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var lines = new List<string>();
            supervisor.Changed += () => { if (supervisor.State == "ready") ready.TrySetResult(); };
            supervisor.Output += (_, line) =>
            {
                if (!line.StartsWith("INPUT ", StringComparison.Ordinal)) return;
                lock (lines)
                {
                    lines.Add(line);
                    if (lines.Count == count * 2) received.TrySetResult();
                }
            };
            var (exe, args) = Fixture.Child("capture-pipes", "full");
            supervisor.Start(new(exe, args, new() { Profile = "full" }));
            try
            {
                await ready.Task.WaitAsync(TimeSpan.FromSeconds(10));
                var sends = Enumerable.Range(1, count).SelectMany(i => new[]
                {
                    Task.Run(async () => { await start.Task; await supervisor.SendMidiCaptureMarkerAsync((ulong)i); }),
                    Task.Run(async () => { await start.Task; await supervisor.SendScreenAsync((ulong)i, "OFF"); })
                }).ToArray();
                start.SetResult();
                await Task.WhenAll(sends).WaitAsync(TimeSpan.FromSeconds(10));
                await received.Task.WaitAsync(TimeSpan.FromSeconds(10));
                await supervisor.StopAsync().WaitAsync(TimeSpan.FromSeconds(10));
                lock (lines)
                {
                    var expected = Enumerable.Range(1, count).SelectMany(i => new[]
                    {
                        $"INPUT S4_MIDI_CAPTURE_MARK {i} issue",
                        $"INPUT S4_SCREEN_CONTROL {i} OFF"
                    }).Append("INPUT S4_BRIDGE_CONTROL 1 STOP").Order().ToArray();
                    Check.True(expected.SequenceEqual(lines.Order()));
                }
                Check.Equal("stopped", supervisor.State);
                Check.True(!supervisor.HasChild);
                await Check.RejectAsync(() => supervisor.SendMidiCaptureMarkerAsync(99));
            }
            finally
            {
                if (supervisor.HasChild)
                {
                    supervisor.CloseInput();
                    await supervisor.Completion.WaitAsync(TimeSpan.FromSeconds(10));
                }
            }
        }));
    }

    private static string Status(object value) =>
        "S4_MIDI_CAPTURE_STATUS " + JsonSerializer.Serialize(value);

    private static string Completion(string state, string path, string? error) =>
        Status(new { version = 1, state, path, error, counts = new
        {
            attempted = 8, queued = 5, written = 3, dropped_full = 1,
            dropped_oversized = 1, dropped_disconnected = 1, dropped_limit = 0, dropped_limit_queued = 0,
            unwritten = 2, truncated = false
        } });

    private static MidiCapture Recording(Fixture fixture, out Guid launch, out string path)
    {
        var capture = new MidiCapture { EnabledForNextLaunch = true };
        path = capture.PrepareLaunch(fixture.Root, "full")!;
        launch = Guid.NewGuid();
        capture.Launched(launch);
        capture.Accept(launch, Status(new { version = 1, state = "started", path, max_file_bytes = 1048576 }));
        Check.Equal("recording", capture.State);
        return capture;
    }
}
