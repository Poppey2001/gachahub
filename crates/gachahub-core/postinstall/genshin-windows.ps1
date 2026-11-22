param(
  [Parameter(Mandatory=$false)][string]$InstallPath = "$env:USERPROFILE\Games\GachaHub\Games\genshin",
  [Parameter(Mandatory=$false)][string]$PrefixPath = ""
)

$exe = Get-ChildItem -Path $InstallPath -Filter "GenshinImpact.exe" -File -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
$exePath = if ($exe) { $exe.FullName } else { "" }

Write-Output "profileVersion=genshin-defaults-v1"
Write-Output "gameId=genshin"
Write-Output "platform=windows"
Write-Output "installPath=$InstallPath"
Write-Output "prefixPath=$PrefixPath"
Write-Output "executablePath=$exePath"
Write-Output "runtime=native"
Write-Output "runnerVersion=Native"
Write-Output "runnerPath="
Write-Output "launchArguments="
Write-Output "environmentVariables="
Write-Output "gamescopeEnabled=false"
Write-Output "gameModeEnabled=false"
Write-Output "mangoHudEnabled=false"
Write-Output "preventSleep=true"
Write-Output "autoUpdate=true"
Write-Output "allowPreloads=true"
Write-Output "updateChannel=stable"
Write-Output "modsEnabled=false"
Write-Output "verifyBeforeLaunch=false"
Write-Output "xxmiEnabled=false"
Write-Output "hasGameMode=false"
Write-Output "hasGamescope=false"
Write-Output "hasMangoHud=false"
Write-Output "hasUmu=false"
Write-Output "warning=Keine Anti-Cheat- oder Schutzmechanismus-Workarounds werden automatisch gesetzt."
if (-not $exe) {
  Write-Output "warning=GenshinImpact.exe wurde im Installationspfad noch nicht gefunden. Der Pfad wird trotzdem als Standard übernommen."
}
