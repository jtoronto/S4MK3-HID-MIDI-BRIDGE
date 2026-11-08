using Avalonia.Controls;
using Avalonia.Controls.Presenters;
using S4Bridge.Diagnostics;

namespace S4Bridge.Views;

public sealed class IncrementalLogView : TextBox
{
    protected override Type StyleKeyOverride => typeof(TextBox);

    public IncrementalLogView()
    {
        IsReadOnly = true;
        IsUndoEnabled = false;
        AcceptsReturn = true;
        // The inner viewer gets the caret request first; do not let an already
        // visible caret pull the surrounding Diagnostics page down to the log.
        AddHandler(RequestBringIntoViewEvent, (_, e) =>
        {
            if (e.TargetObject is TextPresenter) e.Handled = true;
        });
    }
    public void Apply(LogDelta delta)
    {
        if (delta.RemovePrefix == 0 && delta.Append.Length == 0) return;
        var oldLength = Text?.Length ?? 0;
        var start = SelectionStart; var end = SelectionEnd;
        var followTail = start == oldLength && end == oldLength;
        // SelectedText intentionally honors IsReadOnly in Avalonia 11. Temporarily
        // permit this synchronous dispatcher-owned edit; no user input can interleave.
        IsReadOnly = false;
        try
        {
            if (delta.RemovePrefix > 0)
            {
                SelectionStart = 0; SelectionEnd = delta.RemovePrefix; SelectedText = "";
            }
            if (delta.Append.Length > 0)
            {
                SelectionStart = Text?.Length ?? 0; SelectionEnd = SelectionStart; SelectedText = delta.Append;
            }
            if (!followTail)
            {
                SelectionStart = Math.Clamp(start - delta.RemovePrefix, 0, Text?.Length ?? 0);
                SelectionEnd = Math.Clamp(end - delta.RemovePrefix, 0, Text?.Length ?? 0);
            }
        }
        finally { IsReadOnly = true; }
    }
}
