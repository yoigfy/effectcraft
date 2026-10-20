#define MyAppName "TokEffect"
#ifndef MyAppVersion
  #define MyAppVersion "0.12.0"
#endif
#define MyAppExeName "TokEffect.exe"

[Setup]
AppId={{7A3F2E11-6C9B-4D85-9D8F-3B7D65A102E4}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=TokEffect
AppCopyright=Copyright (C) 2026 TokEffect contributors
AppComments=TokEffect video editor and motion graphics
DefaultDirName={autopf}\TokEffect
DefaultGroupName=TokEffect
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=..\..\dist
OutputBaseFilename=TokEffect
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\TokEffect.exe
SetupIconFile=..\..\dist\tokeffect.ico
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#MyAppVersion}
VersionInfoProductName=TokEffect
VersionInfoDescription=TokEffect video editor and motion graphics

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts"; Flags: unchecked

[Files]
Source: "..\..\dist\TokEffect-App.exe"; DestDir: "{app}"; DestName: "TokEffect.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\TokEffect"; Filename: "{app}\TokEffect.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\TokEffect"; Filename: "{app}\TokEffect.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\TokEffect.exe"; Description: "Launch TokEffect"; Flags: postinstall nowait skipifsilent
