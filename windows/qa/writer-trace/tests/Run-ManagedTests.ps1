#Requires -Version 5.1
# With a compiled test apphost, invoke WriterSessionTests directly. A
# PowerShell wrapper supplies the same child-process entrypoint on macOS.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$sources = @('WriterSession.cs','WriterSession.Process.cs','WriterSession.Protocol.cs',
    'TraceDeadline.cs','tests/WriterSessionTests.cs') | ForEach-Object { Join-Path $root $_ }
if ($PSVersionTable.PSEdition -eq 'Desktop') {
    Add-Type -Path $sources -ReferencedAssemblies @(
        'System.dll','System.Core.dll','System.Xml.dll','System.Runtime.Serialization.dll')
} else {
    Add-Type -Path $sources
}
exit [WriterSessionTests]::Main([string[]]$args)
