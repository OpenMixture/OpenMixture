param(
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [string]$BrowserPath = 'C:\Program Files\Google\Chrome\Application\chrome.exe',
    [ValidateRange(1024, 65535)][int]$Port = 9334
)
$ErrorActionPreference = 'Stop'
$browserFile = Get-Item -LiteralPath $BrowserPath
$outputPath = [System.IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $outputPath) { throw 'Use a new launch directory; a reused profile is not default-profile evidence.' }
if (Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue) { throw 'Debugging port is already in use.' }
New-Item -ItemType Directory -Path $outputPath | Out-Null
$profilePath = Join-Path $outputPath 'profile'
# Only profile isolation and local automation transport; no headless, GPU,
# sandbox, feature, blocklist, or first-run overrides. Keep personal profiles intact.
$launchArguments = @("--user-data-dir=`"$profilePath`"", "--remote-debugging-port=$Port", 'about:blank')
$browserProcess = Start-Process -FilePath $browserFile.FullName -ArgumentList $launchArguments -WindowStyle Hidden -PassThru
$observed = Get-CimInstance Win32_Process -Filter "ProcessId=$($browserProcess.Id)"
if (-not $observed) { throw 'Could not observe the launched browser process.' }
$record = [ordered]@{
    schemaVersion = 1
    startedAt = [DateTimeOffset]::Now.ToString('o')
    freshProfile = $true
    executable = $browserFile.FullName
    executableSha256 = (Get-FileHash -LiteralPath $browserFile.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    product = $browserFile.VersionInfo.ProductName
    version = $browserFile.VersionInfo.ProductVersion
    processId = $browserProcess.Id
    arguments = $launchArguments
    observedCommandLine = $observed.CommandLine
    endpoint = "http://127.0.0.1:$Port"
    os = Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber, OSArchitecture
    hostVideoControllers = @(Get-CimInstance Win32_VideoController | Select-Object Name, DriverVersion)
    scope = 'Fresh ordinary desktop profile; only profile isolation and CDP transport switches. Host GPUs are not inferred runtime adapter identities.'
}
$record | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $outputPath 'launch.json') -Encoding utf8
Write-Output "Launch record: $(Join-Path $outputPath 'launch.json')"
