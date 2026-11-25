; GachaHub Windows installer (Inno Setup 6)
; Build:  iscc /DAppVersion=0.16.0 packaging\windows\gachahub.iss
; Installs per user (no admin rights) so GachaHub's self-updater can replace its own exe.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{6F1C2B9E-3D47-4C59-9A3E-5B2D8E7F1A10}
AppName=GachaHub
AppVersion={#AppVersion}
AppVerName=GachaHub {#AppVersion}
AppPublisher=GachaHub contributors
AppPublisherURL=https://github.com/Poppey2001/gachahub
AppSupportURL=https://github.com/Poppey2001/gachahub/issues
AppUpdatesURL=https://github.com/Poppey2001/gachahub/releases
DefaultDirName={localappdata}\Programs\GachaHub
DefaultGroupName=GachaHub
DisableProgramGroupPage=yes
DisableDirPage=auto
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\..\dist
OutputBaseFilename=gachahub-setup
SetupIconFile=..\..\crates\gachahub-app\assets\gachahub.ico
UninstallDisplayIcon={app}\gachahub.exe
UninstallDisplayName=GachaHub
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=force
RestartApplications=no

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\target\release\gachahub.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\GachaHub"; Filename: "{app}\gachahub.exe"
Name: "{autodesktop}\GachaHub"; Filename: "{app}\gachahub.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\gachahub.exe"; Description: "{cm:LaunchProgram,GachaHub}"; Flags: nowait postinstall skipifsilent

[InstallDelete]
; leftovers of the self-updater / a previous setup
Type: files; Name: "{app}\*.old"

[UninstallDelete]
; leftovers of the self-updater
Type: files; Name: "{app}\*.old"

[Code]
// Before copying: stop running GachaHub instances and move the old exe aside.
// Windows allows renaming a running (or locked-for-write) exe, so the new
// file can always be written, even if a process could not be stopped.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
  Exe, Old: String;
begin
  Result := '';
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM gachahub.exe', '', SW_HIDE,
    ewWaitUntilTerminated, ResultCode);
  Sleep(500);
  // stale shortcuts from an earlier install are recreated fresh
  DeleteFile(ExpandConstant('{autoprograms}\GachaHub.lnk'));
  DeleteFile(ExpandConstant('{autodesktop}\GachaHub.lnk'));
  Exe := ExpandConstant('{app}\gachahub.exe');
  Old := Exe + '.old';
  if FileExists(Exe) then
  begin
    DeleteFile(Old);
    if not RenameFile(Exe, Old) then
      Log('could not move old gachahub.exe aside; Inno will retry the copy');
  end;
end;
