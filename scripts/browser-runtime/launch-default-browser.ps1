param(
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [string]$BrowserPath = 'C:\Program Files\Google\Chrome\Application\chrome.exe',
    [ValidateSet('chromium', 'firefox')][string]$Family = 'chromium',
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
if ($Family -eq 'firefox') {
    New-Item -ItemType Directory -Path $profilePath | Out-Null
    $launchArguments = @('-profile', "`"$profilePath`"", "--remote-debugging-port=$Port", 'about:blank')
}
$browserProcess = Start-Process -FilePath $browserFile.FullName -ArgumentList $launchArguments -WindowStyle Hidden -PassThru
$observed = Get-CimInstance Win32_Process -Filter "ProcessId=$($browserProcess.Id)"
if (-not $observed) { throw 'Could not observe the launched browser process.' }
$record = [ordered]@{
    schemaVersion = 1
    browserFamily = $Family
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
    scope = 'Fresh ordinary desktop profile; only profile isolation and loopback CDP/BiDi transport switches. Host GPUs are not inferred runtime adapter identities.'
}
if ($Family -eq 'firefox') {
    # Firefox's launcher creates a browser parent process. Retain both identities.
    $child = $null
    for ($attempt = 0; $attempt -lt 50 -and -not $child; $attempt++) {
        $child = Get-CimInstance Win32_Process -Filter "ParentProcessId=$($browserProcess.Id) AND Name='firefox.exe'" | Select-Object -First 1
        if (-not $child) { Start-Sleep -Milliseconds 100 }
    }
    if (-not $child) { throw 'Could not observe Firefox browser parent process.' }
    $record.browserProcess = @{ processId = $child.ProcessId; parentProcessId = $child.ParentProcessId; commandLine = $child.CommandLine }
    $record.endpoint = "ws://127.0.0.1:$Port/session"
}
$record | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $outputPath 'launch.json') -Encoding utf8
Write-Output "Launch record: $(Join-Path $outputPath 'launch.json')"
