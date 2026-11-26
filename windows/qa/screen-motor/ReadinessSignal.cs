namespace S4UsbCapture
{
    using System;
    using System.IO;
    using System.Threading;

    public sealed class ReadinessSignal : IDisposable
    {
        private readonly AutoResetEvent changed = new AutoResetEvent(false);
        private readonly FileSystemWatcher watcher;
        private readonly object gate = new object();
        private Exception failure;

        public ReadinessSignal(string directory)
        {
            watcher = new FileSystemWatcher(directory, "capture.readiness.json");
            watcher.NotifyFilter = NotifyFilters.FileName | NotifyFilters.LastWrite;
            watcher.Changed += OnChange;
            watcher.Created += OnChange;
            watcher.Renamed += OnChange;
            watcher.Error += delegate(object sender, ErrorEventArgs args)
            {
                lock (gate) { failure = args.GetException(); }
                Signal();
            };
            watcher.EnableRaisingEvents = true;
        }

        private void OnChange(object sender, FileSystemEventArgs args) { Signal(); }
        private void Signal()
        {
            try { changed.Set(); }
            catch (ObjectDisposedException) { }
        }

        public bool Wait(int milliseconds)
        {
            bool signaled = changed.WaitOne(milliseconds);
            lock (gate)
            {
                if (failure != null) throw new IOException("Diagnostic readiness watch failed", failure);
            }
            return signaled;
        }

        public void Dispose()
        {
            watcher.Dispose();
            changed.Dispose();
        }
    }
}
