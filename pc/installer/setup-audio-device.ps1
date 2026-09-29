# Mikey - Audio Endpoint Auto-Setup & Rebranding
# Installs / repairs the virtual audio driver and renames endpoints to "Mikey Mic" and "Mikey Audio Bridge".
# Auto-elevates to Administrator if needed.

param (
    [switch]$Silent = $false
)

$IsAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

$scriptPath = if ($PSCommandPath) { $PSCommandPath } else { $MyInvocation.MyCommand.Path }
if (-not $scriptPath -or -not (Test-Path $scriptPath)) {
    $scriptPath = Join-Path $PSScriptRoot "setup-audio-device.ps1"
}

if (-not $IsAdmin) {
    if (-not $Silent) {
        Write-Host "[mikey] Administrator privileges required to configure Mikey Mic." -ForegroundColor Yellow
        Write-Host "[mikey] Requesting UAC elevation..." -ForegroundColor Cyan
        try {
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = "powershell.exe"
            $psi.Arguments = "-ExecutionPolicy Bypass -NoProfile -File `"$scriptPath`""
            $psi.Verb = "runas"
            $psi.UseShellExecute = $true
            $proc = [System.Diagnostics.Process]::Start($psi)
            if ($proc) {
                $proc.WaitForExit()
                exit $proc.ExitCode
            }
            exit 0
        } catch {
            Write-Host ""
            Write-Host "[mikey] Auto-elevation could not open prompt: $_" -ForegroundColor Yellow
            Write-Host "[mikey] To complete audio device setup:" -ForegroundColor Cyan
            Write-Host "       1. In File Explorer, go to: e:\Programs\mikey\pc" -ForegroundColor Cyan
            Write-Host "       2. Right-click 'setup-mic.cmd' and select 'Run as administrator'" -ForegroundColor Cyan
            Write-Host ""
            exit 1
        }
    } else {
        Write-Error "[mikey] Administrator privileges required to setup audio device."
        exit 1
    }
}

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "  Mikey Audio Setup: Configuring Mikey Mic...  " -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan

# 1. Check if driver is already running healthy
$pnp = Get-PnpDevice -Class Media -ErrorAction SilentlyContinue | Where-Object { 
    $_.InstanceId -like "*MEDIA\0003*" -or $_.FriendlyName -like "*CABLE*" -or $_.FriendlyName -like "*VB-Audio*" -or $_.FriendlyName -like "*Mikey*"
} | Select-Object -First 1

$needsInstall = $true
if ($pnp -and $pnp.Status -eq "OK") {
    Write-Host "[mikey] Found running audio driver device: $($pnp.FriendlyName)" -ForegroundColor Green
    $needsInstall = $false
}

if ($needsInstall) {
    Write-Host "[mikey] Installing / repairing driver..." -ForegroundColor Cyan

    # A. Check DriverStore INF
    $dsInf = Get-ChildItem -Path "C:\Windows\System32\DriverStore\FileRepository" -Filter "vbmmecable64_win10.inf" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($dsInf) {
        Write-Host "[mikey] Adding driver from DriverStore: $($dsInf.FullName)" -ForegroundColor Cyan
        & pnputil.exe /add-driver "$($dsInf.FullName)" /install | Out-Null
    }

    # B. Run bundled setup executable
    $setupExe = "C:\Program Files\VB\CABLE\VBCABLE_Setup_x64.exe"
    if (Test-Path $setupExe) {
        Write-Host "[mikey] Running driver installer: $setupExe" -ForegroundColor Cyan
        $proc = Start-Process -FilePath $setupExe -ArgumentList "-i", "-h" -PassThru
        $proc.WaitForExit(10000)
    }

    # C. Restart device if error persists
    Start-Sleep -Seconds 1
    $dev = Get-PnpDevice -Class Media -ErrorAction SilentlyContinue | Where-Object { 
        $_.InstanceId -like "*MEDIA\0003*" -or $_.FriendlyName -like "*CABLE*" -or $_.FriendlyName -like "*VB-Audio*"
    } | Select-Object -First 1
    if ($dev) {
        & pnputil.exe /restart-device "$($dev.InstanceId)" | Out-Null
    }

    Start-Sleep -Seconds 1
}

# 2. Rebrand Audio Endpoints in Registry to "Mikey Mic" and "Mikey Audio Bridge"
$FriendlyNameProp = "{a45c254e-df1c-4efd-8020-67d146a850e0},2"
$DeviceDescProp   = "{b3f8fa53-0004-438e-9003-51a46e139bfc},6"
$InterfaceProp    = "{b3f8fa53-0004-438e-9003-51a46e139bfc},2"

# 2A. Capture Endpoints (Microphones) -> "Mikey Mic"
$capturePath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture"
if (Test-Path $capturePath) {
    Get-ChildItem -Path $capturePath -Recurse -ErrorAction SilentlyContinue | Where-Object {
        $_.Property -contains $FriendlyNameProp
    } | ForEach-Object {
        $keyPath = $_.PSPath
        $props = Get-ItemProperty -Path $keyPath -ErrorAction SilentlyContinue
        $currentName = $props.$FriendlyNameProp
        $deviceDesc = $props.$DeviceDescProp
        $interface = $props.$InterfaceProp

        if ($currentName -like "*CABLE*" -or $currentName -like "*VB-Audio*" -or $deviceDesc -like "*VB-Audio*" -or $interface -like "*0003*") {
            Write-Host "[mikey] Rebranding Capture Endpoint '$currentName' -> 'Mikey Mic'..." -ForegroundColor Green
            Set-ItemProperty -Path $keyPath -Name $FriendlyNameProp -Value "Mikey Mic" -Force
        }
    }
}

# 2B. Clean up Render Endpoints (Speakers)
# The user wants ONLY their own physical speakers visible for audio output.
$renderPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render"
if (Test-Path $renderPath) {
    Get-ChildItem -Path $renderPath -Recurse -ErrorAction SilentlyContinue | Where-Object {
        $_.Property -contains $FriendlyNameProp
    } | ForEach-Object {
        $keyPath = $_.PSPath
        $props = Get-ItemProperty -Path $keyPath -ErrorAction SilentlyContinue
        $currentName = $props.$FriendlyNameProp
        $deviceDesc = $props.$DeviceDescProp
        $interface = $props.$InterfaceProp
        $parentKey = Split-Path $keyPath

        # 1. Disable orphan AudioRelay virtual speaker so it disappears from output list
        if ($currentName -like "*Virtual Speaker*" -or $deviceDesc -like "*AudioRelay*") {
            Write-Host "[mikey] Disabling orphan output '$currentName'..." -ForegroundColor Yellow
            Set-ItemProperty -Path $parentKey -Name "DeviceState" -Value 268435458 -Force -ErrorAction SilentlyContinue
        }

        # 2. Disable duplicate "Speakers (VB-Audio Virtual Cable)" so user only sees real speakers
        if ($currentName -like "*Speakers*" -and ($deviceDesc -like "*VB-Audio*" -or $interface -like "*0003*")) {
            Write-Host "[mikey] Disabling extra virtual speaker '$currentName'..." -ForegroundColor Yellow
            Set-ItemProperty -Path $parentKey -Name "DeviceState" -Value 268435458 -Force -ErrorAction SilentlyContinue
        }

        # 3. Rebrand the internal bridge endpoint to "Mikey Mic Bridge"
        if (($currentName -like "*CABLE In*" -or $currentName -like "*CABLE*" -or $currentName -like "*Mikey*") -and $currentName -notlike "*Realtek*" -and $currentName -notlike "*Speakers*") {
            Write-Host "[mikey] Setting Render Bridge '$currentName' -> 'Mikey Mic Bridge'..." -ForegroundColor Green
            Set-ItemProperty -Path $keyPath -Name $FriendlyNameProp -Value "Mikey Mic Bridge" -Force
        }
    }
}

# 3. Restart Windows Audio Service to apply changes
Write-Host "[mikey] Refreshing Windows Audio Service..." -ForegroundColor Cyan
try {
    Restart-Service -Name "Audiosrv" -Force -ErrorAction Stop
    Write-Host "[mikey] Windows Audio Service refreshed." -ForegroundColor Green
    # Gently notify the shell that device associations changed (do NOT kill explorer -
    # that breaks Windows 11 Quick Settings flyouts for Wi-Fi, Sound, Bluetooth)
    $shNotify = @'
using System;
using System.Runtime.InteropServices;
public class ShellNotify {
    [DllImport("shell32.dll")]
    public static extern void SHChangeNotify(int wEventId, int uFlags, IntPtr dwItem1, IntPtr dwItem2);
    public static void Refresh() { SHChangeNotify(0x08000000, 0, IntPtr.Zero, IntPtr.Zero); }
}
'@
    if (-not ([System.Management.Automation.PSTypeName]'ShellNotify').Type) {
        Add-Type -TypeDefinition $shNotify -ErrorAction SilentlyContinue
    }
    [ShellNotify]::Refresh()
} catch {
    Write-Warning "[mikey] Could not restart Audiosrv automatically: $_"
}

# 3B. Preserve User's Physical Speaker Output & Set Mikey Mic as Default Input
try {
    $cSharp = @'
using System;
using System.Runtime.InteropServices;

[Guid("F8679F50-850A-41CF-9C72-430F290290C8"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IPolicyConfig {
    void GetMixFormat();
    void GetDeviceFormat();
    void ResetDeviceFormat();
    void SetDeviceFormat();
    void GetProcessingPeriod();
    void SetProcessingPeriod();
    void GetShareMode();
    void SetShareMode();
    void GetPropertyValue();
    void SetPropertyValue();
    void SetDefaultEndpoint(string wszDeviceId, int eRole);
    void SetEndpointVisibility();
}

[Guid("870AF99C-171D-4F9E-AF0D-E63DF40C2BC9")]
public class CPolicyConfigClient {}

public class AudioDeviceHelper {
    public static int SetDefault(string deviceId) {
        try {
            IPolicyConfig policy = (IPolicyConfig)new CPolicyConfigClient();
            policy.SetDefaultEndpoint(deviceId, 0); // eConsole
            policy.SetDefaultEndpoint(deviceId, 1); // eMultimedia
            policy.SetDefaultEndpoint(deviceId, 2); // eCommunications
            return 0;
        } catch {
            return -1;
        }
    }
}
'@
    if (-not ([System.Management.Automation.PSTypeName]'AudioDeviceHelper').Type) {
        Add-Type -TypeDefinition $cSharp -ErrorAction SilentlyContinue
    }

    # Ensure Physical Speaker is the Default Output (Render)
    $physicalSpeaker = Get-ChildItem -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render" -ErrorAction SilentlyContinue | Where-Object {
        (Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue).DeviceState -eq 1
    } | Where-Object {
        $props = Get-ItemProperty ($_.PSPath + "\Properties") -ErrorAction SilentlyContinue
        $name = $props.$FriendlyNameProp
        $desc = $props.$DeviceDescProp
        $name -notlike "*Mikey*" -and $name -notlike "*CABLE*" -and $name -notlike "*VB-Audio*" -and $desc -notlike "*VB-Audio*"
    } | Select-Object -First 1

    if ($physicalSpeaker) {
        $spkName = (Get-ItemProperty ($physicalSpeaker.PSPath + "\Properties") -ErrorAction SilentlyContinue).$FriendlyNameProp
        [AudioDeviceHelper]::SetDefault($physicalSpeaker.PSChildName) | Out-Null
        Write-Host "[mikey] Speaker Output: '$spkName' remains your default output (Unchanged)" -ForegroundColor Green
    }

    # Set Mikey Mic as Default Recording Device (Capture)
    $mikeyMic = Get-ChildItem -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture" -ErrorAction SilentlyContinue | Where-Object {
        (Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue).DeviceState -eq 1
    } | Where-Object {
        $props = Get-ItemProperty ($_.PSPath + "\Properties") -ErrorAction SilentlyContinue
        $name = $props.$FriendlyNameProp
        $name -like "*Mikey*" -or $name -like "*CABLE Output*"
    } | Select-Object -First 1

    if ($mikeyMic) {
        [AudioDeviceHelper]::SetDefault($mikeyMic.PSChildName) | Out-Null
        Write-Host "[mikey] Microphone Input: 'Mikey Mic' set as default recording input" -ForegroundColor Green
    }
} catch {
    Write-Warning "[mikey] Default endpoint assignment skipped: $_"
}

# 4. Final verification
Start-Sleep -Seconds 1
$pnpFinal = Get-PnpDevice -Class Media -ErrorAction SilentlyContinue | Where-Object { 
    $_.InstanceId -like "*MEDIA\0003*" -or $_.FriendlyName -like "*CABLE*" -or $_.FriendlyName -like "*VB-Audio*" -or $_.FriendlyName -like "*Mikey*"
} | Select-Object -First 1

if ($pnpFinal -and $pnpFinal.Status -eq "OK") {
    Write-Host ""
    Write-Host "[mikey] SUCCESS! Mikey Mic is installed, healthy, and ready!" -ForegroundColor Green
    Write-Host "[mikey] Windows applications will now detect 'Mikey Mic' as an input device." -ForegroundColor Green
} else {
    Write-Host "[mikey] Setup finished. Status: $($pnpFinal.Status)" -ForegroundColor Yellow
}
