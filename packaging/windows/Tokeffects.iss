#define MyAppName "Tokeffects"
#ifndef MyAppVersion
  #define MyAppVersion "0.12.0"
#endif
#define MyAppExeName "Tokeffects.exe"

[Setup]
AppId={{7A3F2E11-6C9B-4D85-9D8F-3B7D65A102E4}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=Tokeffects
AppCopyright=Copyright (C) 2026 Tokeffects contributors
AppComments=Tokeffects video editor and motion graphics
DefaultDirName={autopf}\Tokeffects
DefaultGroupName=Tokeffects
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=..\..\dist
OutputBaseFilename=Tokeffects
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\Tokeffects.exe
SetupIconFile=..\..\dist\tokeffects.ico
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#MyAppVersion}
VersionInfoProductName=Tokeffects
VersionInfoDescription=Tokeffects video editor and motion graphics

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts"; Flags: unchecked

[Files]
Source: "..\..\dist\Tokeffects-App.exe"; DestDir: "{app}"; DestName: "Tokeffects.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Tokeffects"; Filename: "{app}\Tokeffects.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\Tokeffects"; Filename: "{app}\Tokeffects.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\Tokeffects.exe"; Description: "Launch Tokeffects"; Flags: postinstall nowait skipifsilent
