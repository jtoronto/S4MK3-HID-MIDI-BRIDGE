namespace S4UsbCapture
{
    using System;

    public sealed class CaptureResult
    {
        public bool Success { get; internal set; }
        public bool HeaderReady { get; internal set; }
        public bool ForcedTermination { get; internal set; }
        public bool ParentLost { get; internal set; }
        public bool UnexpectedExit { get; internal set; }
        public int ExitCode { get; internal set; }
        public string Error { get; internal set; }
        public long BytesWritten { get; internal set; }
        public DateTime StartedUtc { get; internal set; }
        public DateTime EndedUtc { get; internal set; }
        public bool UsbPacketLossKnown { get { return false; } }
    }
}
