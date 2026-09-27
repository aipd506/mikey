; Mikey Windows Installer (Inno Setup Script)
; Builds a lightweight installer for Windows 10/11 that installs mikey.exe,
; sets up firewall rules for TCP :7653 and UDP :7654 (private networks only),
; and registers optional autostart.

#define MyAppName "Mikey"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Mikey Contributors"
#define MyAppURL "https://github.com/yashthorat7/mikey"
#define MyAppExeName "mikey.exe"

[Setup]
AppId={{9F3B6E8C-8F74-4C75-A1E2-93D0F8C56A10}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputBaseFilename=Mikey-Setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "Start Mikey automatically on Windows login"; GroupDescription: "Startup:"

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "setup-audio-device.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\setup-mic.cmd"; DestDir: "{app}"; Flags: ignoreversion
; Optional bundled driver binaries (ignored if not present in installer folder at build time)
Source: "VBCABLE_Setup_x64.exe"; DestDir: "{tmp}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "softcam.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "softcam_installer.exe"; DestDir: "{tmp}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Mikey"; ValueData: """{app}\{#MyAppExeName}"""; Tasks: autostart; Flags: uninsdeletevalue

[Run]
; Add Windows Firewall rules on private profiles for TCP 7653 and UDP 7654 per connection-levels.md
Filename: "netsh"; Parameters: "advfirewall firewall add rule name=""Mikey TCP"" dir=in action=allow protocol=TCP localport=7653 profile=private"; Flags: runhidden
Filename: "netsh"; Parameters: "advfirewall firewall add rule name=""Mikey UDP Beacon"" dir=in action=allow protocol=UDP localport=7654 profile=private"; Flags: runhidden
; Silently install VB-Cable if bundled and not yet installed
Filename: "{tmp}\VBCABLE_Setup_x64.exe"; Parameters: "-i -h"; Flags: runhidden skipifdoesntexist
; Run audio device setup script to brand endpoints as "Mikey Mic"
Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -NoProfile -File ""{app}\setup-audio-device.ps1"" -Silent"; Flags: runhidden
; Register softcam virtual camera DirectShow filter if bundled
Filename: "{tmp}\softcam_installer.exe"; Parameters: "/install ""{app}\softcam.dll"""; Flags: runhidden skipifdoesntexist
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "netsh"; Parameters: "advfirewall firewall delete rule name=""Mikey TCP"""; Flags: runhidden
Filename: "netsh"; Parameters: "advfirewall firewall delete rule name=""Mikey UDP Beacon"""; Flags: runhidden
