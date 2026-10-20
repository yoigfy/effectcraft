#define MyAppName "TokCraft"
#ifndef MyAppVersion
  #define MyAppVersion "0.12.0"
#endif
#define MyAppExeName "TokCraft.exe"

[Setup]
AppId={{7A3F2E11-6C9B-4D85-9D8F-3B7D65A102E4}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=TokCraft
AppCopyright=Copyright (C) 2026 TokCraft contributors
AppComments=TokCraft video editor and motion graphics
DefaultDirName={autopf}\TokCraft
DefaultGroupName=TokCraft
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=..\..\dist
OutputBaseFilename=TokCraft
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\TokCraft.exe
SetupIconFile=..\..\dist\tokcraft.ico
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#MyAppVersion}
VersionInfoProductName=TokCraft
VersionInfoDescription=TokCraft video editor and motion graphics

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts"; Flags: unchecked

[Files]
Source: "..\..\dist\TokCraft-App.exe"; DestDir: "{app}"; DestName: "TokCraft.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\TokCraft"; Filename: "{app}\TokCraft.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\TokCraft"; Filename: "{app}\TokCraft.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\TokCraft.exe"; Description: "Launch TokCraft"; Flags: postinstall nowait skipifsilent
