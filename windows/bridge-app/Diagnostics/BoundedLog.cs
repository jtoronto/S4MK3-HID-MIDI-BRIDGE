namespace S4Bridge.Diagnostics;

public sealed record LogDelta(long Cursor, int RemovePrefix, string Append);
public sealed class BoundedLog
{
    public const int Capacity = 60000;
    private readonly object gate = new();
    private string text = "";
    private long end;
    public string Text { get { lock (gate) return text; } }
    public void Append(string value)
    {
        lock (gate)
        {
            var addition = value + "\n";
            end = checked(end + addition.Length);
            text += addition;
            if (text.Length > Capacity)
            {
                var start = text.Length - Capacity;
                if (char.IsLowSurrogate(text[start])) start++;
                text = text[start..];
            }
        }
    }
    public LogDelta ReadSince(long cursor, int displayedLength)
    {
        lock (gate)
        {
            var count = end - cursor;
            if (count < 0 || count >= text.Length)
                return new(end, displayedLength, text);
            var added = checked((int)count);
            return new(end, Math.Max(0, displayedLength + added - text.Length),
                added == 0 ? "" : text[^added..]);
        }
    }
}
