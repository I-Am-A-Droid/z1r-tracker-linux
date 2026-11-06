; Inno Setup script for the Windows installer of Z-Tracker (the Linux/Windows/macOS rebuild).
; Build: iscc windows\installer.iss   (expects target\x86_64-pc-windows-gnu\release\z1r-tracker.exe, or pass
; /DExeDir=<folder with z1r-tracker.exe>). Installs for the current user only, so no administrator rights are needed.

#ifndef ExeDir
  #define ExeDir "..\target\x86_64-pc-windows-gnu\release"
#endif

[Setup]
AppId={{6E4A3B8C-1D2F-4E5A-9B7C-2F1E8D3A4C5B}
AppName=Z-Tracker for Zelda 1 Randomizer
AppVersion=2.1.0
AppPublisher=Z-Tracker rebuild
DefaultDirName={localappdata}\Programs\Z-Tracker
DefaultGroupName=Z-Tracker
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=z1r-tracker-setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayName=Z-Tracker for Zelda 1 Randomizer
UninstallDisplayIcon={app}\z1r-tracker.exe

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:"

[Files]
Source: "{#ExeDir}\z1r-tracker.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\Z-Tracker"; Filename: "{app}\z1r-tracker.exe"
Name: "{group}\Uninstall Z-Tracker"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Z-Tracker"; Filename: "{app}\z1r-tracker.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\z1r-tracker.exe"; Description: "Start Z-Tracker now"; Flags: nowait postinstall skipifsilent
