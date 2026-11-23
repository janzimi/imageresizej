#define AppName "ImageResizerJ"
#define AppVersion "0.3.3"
#ifndef StageDir
  #define StageDir "C:\RustTarget\stage"
#endif
#define Clsid "{7D4B3C9E-5A1F-4E82-9B6A-3C8D1F2E4A50}"

[Setup]
AppId={{7D4B3C9E-5A1F-4E82-9B6A-3C8D1F2E4A50}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppName}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
OutputDir=..\..\dist
OutputBaseFilename=ImageResizerJ_Setup_{#AppVersion}
SetupIconFile=..\..\assets\icon.ico
UninstallDisplayIcon={app}\imageresizej.exe
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ShowLanguageDialog=auto
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[CustomMessages]
english.ShortcutComment=Image compression settings
russian.ShortcutComment=Настройки сжатия изображений
english.InstallCertMsg=Installing Explorer extension certificate...
russian.InstallCertMsg=Установка сертификата расширения Проводника...
english.RegisterMenuMsg=Registering Windows 11 context menu...
russian.RegisterMenuMsg=Регистрация контекстного меню Windows 11...
english.LaunchSettings=Open ImageResizerJ settings
russian.LaunchSettings=Открыть настройки ImageResizerJ

[Files]
Source: "{#StageDir}\imageresizej.exe"; DestDir: "{app}"; Flags: ignoreversion restartreplace
Source: "{#StageDir}\resizer-worker.exe"; DestDir: "{app}"; Flags: ignoreversion restartreplace
Source: "{#StageDir}\resizer_win_shell.dll"; DestDir: "{app}"; Flags: ignoreversion restartreplace uninsrestartdelete
Source: "{#StageDir}\ImageResizerJ.msix"; DestDir: "{app}"; Flags: ignoreversion; Check: IsWindows11
Source: "{#StageDir}\ImageResizerJ.cer"; DestDir: "{app}"; Flags: ignoreversion; Check: IsWindows11

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\imageresizej.exe"; Comment: "{cm:ShortcutComment}"

[Registry]
Root: HKLM; Subkey: "SOFTWARE\Classes\CLSID\{{#Clsid}"; ValueType: string; ValueData: "ImageResizerJ Shell Extension"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\CLSID\{{#Clsid}\InprocServer32"; ValueType: string; ValueData: "{app}\resizer_win_shell.dll"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\CLSID\{{#Clsid}\InprocServer32"; ValueType: string; ValueName: "ThreadingModel"; ValueData: "Apartment"; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpg\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpg\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpg\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpeg\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpeg\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpeg\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpe\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpe\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jpe\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jfif\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jfif\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.jfif\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.png\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.png\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.png\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.webp\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.webp\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.webp\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.heic\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.heic\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.heic\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.heif\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.heif\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.heif\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.hif\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.hif\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.hif\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.avif\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.avif\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.avif\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.bmp\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.bmp\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.bmp\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.tif\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.tif\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.tif\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.tiff\shell\ImageResizerJ"; ValueType: string; ValueName: "ExplorerCommandHandler"; ValueData: "{{#Clsid}"; Flags: uninsdeletekey; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.tiff\shell\ImageResizerJ"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Check: IsWindows10OrOlder
Root: HKLM; Subkey: "SOFTWARE\Classes\SystemFileAssociations\.tiff\shell\ImageResizerJ"; ValueType: string; ValueName: "CommandStateSync"; ValueData: ""; Check: IsWindows10OrOlder

[Run]
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""Import-Certificate -FilePath '{code:GetPsAppPath}\ImageResizerJ.cer' -CertStoreLocation 'Cert:\LocalMachine\Root' | Out-Null"""; StatusMsg: "{cm:InstallCertMsg}"; Flags: runhidden waituntilterminated; Check: IsWindows11
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""Get-AppxPackage -Name ImageResizerJ -ErrorAction SilentlyContinue | Remove-AppxPackage -ErrorAction SilentlyContinue; Add-AppxPackage -Path '{code:GetPsAppPath}\ImageResizerJ.msix' -ExternalLocation '{code:GetPsAppPath}'"""; StatusMsg: "{cm:RegisterMenuMsg}"; Flags: runhidden waituntilterminated runasoriginaluser; Check: IsWindows11
Filename: "{app}\imageresizej.exe"; Description: "{cm:LaunchSettings}"; Flags: nowait postinstall skipifsilent runasoriginaluser

[UninstallRun]
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""Get-AppxPackage -AllUsers -Name ImageResizerJ -ErrorAction SilentlyContinue | Remove-AppxPackage -AllUsers -ErrorAction SilentlyContinue; Get-AppxPackage -Name ImageResizerJ -ErrorAction SilentlyContinue | Remove-AppxPackage -ErrorAction SilentlyContinue"""; RunOnceId: "RemoveAppx"; Flags: runhidden waituntilterminated; Check: IsWindows11
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""Get-ChildItem Cert:\LocalMachine\Root | Where-Object {{ $_.Subject -eq 'CN=ImageResizerJ' } | Remove-Item -ErrorAction SilentlyContinue"""; RunOnceId: "RemoveCert"; Flags: runhidden waituntilterminated; Check: IsWindows11

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\ImageResizerJ\data\webview"
Type: filesandordirs; Name: "{localappdata}\ImageResizerJ\data\webview-toast"

[Code]
function IsWindows11: Boolean;
var
  Version: TWindowsVersion;
begin
  GetWindowsVersionEx(Version);
  Result := (Version.Major > 10) or ((Version.Major = 10) and (Version.Build >= 22000));
end;

function IsWindows10OrOlder: Boolean;
begin
  Result := not IsWindows11;
end;

function GetPsAppPath(Param: String): String;
begin
  Result := ExpandConstant('{app}');
  StringChangeEx(Result, '''', '''''', True);
end;