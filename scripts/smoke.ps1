# 进程级冒烟回归：托盘常驻零 webview / 二次启动重建 / 关窗销毁
# 前置：先构建 release（cargo build --release --features custom-protocol）
# 用法: .\scripts\smoke.ps1 [-ExePath 路径]
param(
    [string]$ExePath = (Join-Path $PSScriptRoot "..\src-tauri\target\release\PowerPlan.exe")
)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path $ExePath)) { Write-Error "找不到 $ExePath，请先构建 release"; exit 1 }

Add-Type -Namespace Smoke -Name Win -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lp);
[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
[DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr hwnd, System.Text.StringBuilder sb, int max);
[DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
[DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wp, IntPtr lp);
public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lp);
public static IntPtr FindMain(uint targetPid) {
  IntPtr found = IntPtr.Zero;
  EnumWindows((hwnd, lp) => {
    uint pid; GetWindowThreadProcessId(hwnd, out pid);
    if (pid == targetPid) {
      var sb = new System.Text.StringBuilder(256);
      GetWindowTextW(hwnd, sb, 256);
      if (sb.ToString() == "PowerPlan" && IsWindowVisible(hwnd)) { found = hwnd; return false; }
    }
    return true;
  }, IntPtr.Zero);
  return found;
}
'@

function Get-WebviewCount([int]$ProcId) {
  @(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" |
    Where-Object ParentProcessId -eq $ProcId).Count
}

$failures = @()
function Check([string]$Name, [bool]$Condition) {
  if ($Condition) { Write-Host "  [通过] $Name" }
  else { Write-Host "  [失败] $Name"; $script:failures += $Name }
}

Stop-Process -Name PowerPlan -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# 1. 静默启动：托盘常驻零 webview
$a = Start-Process -FilePath $ExePath -ArgumentList '--silent' -PassThru
Start-Sleep -Seconds 4
$a.Refresh()
$wv1 = Get-WebviewCount $a.Id
Check "静默启动进程存活" (-not $a.HasExited)
Check "静默启动零 webview" ($wv1 -eq 0)

# 2. 二次启动：单实例生效，主窗口按需创建
$b = Start-Process -FilePath $ExePath -PassThru
Start-Sleep -Seconds 8
$hwnd = [Smoke.Win]::FindMain($a.Id)
$b.Refresh()
$wv2 = Get-WebviewCount $a.Id
Check "二次启动新实例自动退出" $b.HasExited
Check "主窗口按需创建且可见" ($hwnd -ne [IntPtr]::Zero)
Check "重建后 webview 存在" ($wv2 -eq 1)

# 3. 关闭主窗口：保存几何后销毁 webview，应用保活
$null = [Smoke.Win]::PostMessageW($hwnd, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)
Start-Sleep -Seconds 3
$a.Refresh()
$wv3 = Get-WebviewCount $a.Id
Check "关闭后进程存活（托盘保活）" (-not $a.HasExited)
Check "关闭后 webview 销毁" ($wv3 -eq 0)

Stop-Process -Name PowerPlan -Force -ErrorAction SilentlyContinue

Write-Host ""
if ($failures.Count -gt 0) {
  Write-Error "冒烟失败：$($failures -join '；')"
  exit 1
}
Write-Host "冒烟全部通过"
