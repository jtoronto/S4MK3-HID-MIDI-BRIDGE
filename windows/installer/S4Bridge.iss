#ifndef PayloadDir
  #error "Pass the verified portable ZIP payload directory."
#endif
#ifndef BridgeVersion
  #error "Pass the bridge version."
#endif
#define ProductName "S4 MK3 Bridge"
#define ProductId "{7F451A46-1F48-4E36-AC4A-147C5E22BDE3}"
#define UninstallKey "Software\Microsoft\Windows\CurrentVersion\Uninstall\" + ProductId + "_is1"

[Setup]
AppId={{7F451A46-1F48-4E36-AC4A-147C5E22BDE3}
AppName={#ProductName}
AppVersion={#BridgeVersion}
DefaultDirName={userpf}\{#ProductName}
UsePreviousAppDir=no
DisableDirPage=yes
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\S4Bridge.exe
SetupIconFile={#PayloadDir}\Resources\AppIcon.ico
PrivilegesRequired=lowest
ArchitecturesAllowed=x64os
ArchitecturesInstallIn64BitMode=x64os
MinVersion=10.0.26200
CloseApplications=no
RestartApplications=no
OutputBaseFilename=S4-MK3-Bridge-{#BridgeVersion}-win-x64-Setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
InfoBeforeFile=setup-info.txt

[Files]
Source: "{#PayloadDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{userprograms}\{#ProductName}"; Filename: "{app}\S4Bridge.exe"; WorkingDir: "{app}"

[Code]
function CreateFile(Path: string; Access, Share: Cardinal; Security: LongWord;
  Disposition, Flags: Cardinal; Template: LongWord): Integer;
  external 'CreateFileW@kernel32.dll stdcall';
function CloseHandle(Handle: Integer): Boolean;
  external 'CloseHandle@kernel32.dll stdcall';

const
  QuitMessage = 'Stop the bridge and quit S4 MK3 Bridge from its tray menu before continuing.';

function BridgeIsClosed: Boolean;
var
  Handle, ErrorCode: Integer;
begin
  { The host keeps this file open exclusively for its entire lifetime. }
  Handle := CreateFile(ExpandConstant('{localappdata}\S4 MK3 Bridge\.instance.lock'),
    $80000000, 7, 0, 3, 0, 0);
  ErrorCode := DLLGetLastError;
  if Handle = -1 then
    Result := (ErrorCode = 2) or (ErrorCode = 3)
  else
  begin
    CloseHandle(Handle);
    Result := True;
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): string;
var
  InstallPath, PriorPath: string;
  Entry: TFindRec;
begin
  Result := '';
  if not BridgeIsClosed then
  begin
    Result := QuitMessage;
    Exit;
  end;
  InstallPath := ExpandConstant('{userpf}\{#ProductName}');
  if CompareText(RemoveBackslashUnlessRoot(WizardDirValue), InstallPath) <> 0 then
  begin
    Result := 'Setup installs only to the current user''s Programs folder. Use the ZIP for a different location.';
    Exit;
  end;
  if RegQueryStringValue(HKCU, '{#UninstallKey}', 'InstallLocation', PriorPath) and
     (CompareText(RemoveBackslashUnlessRoot(PriorPath), InstallPath) = 0) then
    Exit;
  if FindFirst(AddBackslash(InstallPath) + '*', Entry) then
  begin
    try
      repeat
        if (Entry.Name <> '.') and (Entry.Name <> '..') then
        begin
          Result := 'The install folder already contains files from another copy. Move that copy aside or use the ZIP.';
          Exit;
        end;
      until not FindNext(Entry);
    finally
      FindClose(Entry);
    end;
  end;
end;

function InitializeUninstall: Boolean;
begin
  Result := BridgeIsClosed;
  if not Result then
    SuppressibleMsgBox(QuitMessage, mbError, MB_OK, IDOK);
end;
