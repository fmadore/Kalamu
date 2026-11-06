<#
Times Kalamu's offline OCR models on this PC, on the processor and on each device the
graphics-processor builds find, with the arguments the app gives (crates/offline/src/vision.rs),
so a decision about which device the app may use rests on measurements
(docs/decisions/2026-11-06-ocr-graphics-processor.md).

Downloads into -Work (default: %TEMP%\kalamu-ocr-bench) the llama.cpp builds pinned in
scripts/llama.json, the two models at their pinned revisions and the sample page, each checked
against its SHA-256; about 2.6 GB, kept for a second run. Needs nothing installed: Windows'
curl.exe and tar.exe do the work. Run in PowerShell, from anywhere:

  powershell -ExecutionPolicy Bypass -File bench-ocr-devices.ps1

or, without the repository:

  irm https://raw.githubusercontent.com/fmadore/Kalamu/main/scripts/bench-ocr-devices.ps1 -OutFile bench.ps1
  powershell -ExecutionPolicy Bypass -File bench.ps1

It prints one line per run (seconds, whether the page's title was read, blocks placed) and keeps
each answer and the program's log in -Work\out. A wrong answer is as important as a slow one.
#>
param(
	[string]$Work = (Join-Path $env:TEMP 'kalamu-ocr-bench'),
	# Also time the OpenCL test build (Windows ARM64 only), from a llama.yml run's artifact
	# unpacked into this folder.
	[string]$OpenClDir = '',
	# The models to time: lightonocr, glm-ocr or both.
	[string[]]$Models = @('lightonocr', 'glm-ocr'),
	# Time the processor build only.
	[switch]$ProcessorOnly
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# As scripts/llama.json and crates/offline/src/vision.rs pin them.
$release = 'llama-b12110-2'
$tag = 'b12110'
$archives = @{
	'x86_64-pc-windows-msvc'  = @{
		llama  = '7dfd4ea94cf50157b938e3684ba7482a70fc6c579734a6aaabd4dd35983b23cb'
		vulkan = '64fdb1176a9a15612df04cfc3f8dc5562652d102c6cb3bdb212f632ce438d737'
	}
	'aarch64-pc-windows-msvc' = @{
		llama  = '2da8f80761be508638cf02a03000f8efb6142a12255c7399d689b5f364349c90'
		vulkan = '78ad5a036359f1ffcf5c87ce2cde339a6d8c32ecdc1cad2a51a4e32fdc75bce5'
	}
}
$modelFiles = @(
	@{ repo = 'prithivMLmods/LightOnOCR-3-1B-GGUF'; rev = 'ff5c331ec2cfee9de1c8054c21ccf88b8c5b0faa'; file = 'LightOnOCR-3-1B.Q8_0.gguf'; sha = '2cf9771aaf236a1c25f389a344510c9ad4daa863e5fdd82651cf7d7834fede1c' },
	@{ repo = 'prithivMLmods/LightOnOCR-3-1B-GGUF'; rev = 'ff5c331ec2cfee9de1c8054c21ccf88b8c5b0faa'; file = 'LightOnOCR-3-1B.mmproj-q8_0.gguf'; sha = '7bbf40e6f3e91626843fdcbba67c7fe774821f3f0c537dcc8c769026a528839a' },
	@{ repo = 'ggml-org/GLM-OCR-GGUF'; rev = '65a42de1148dbed2297e922b5dbc7d9b70c36578'; file = 'GLM-OCR-Q8_0.gguf'; sha = '45bc244a6446aff850521dc41f18bc8d7105ad5f0c2c8c28af04e7cc4f4d50b1' },
	@{ repo = 'ggml-org/GLM-OCR-GGUF'; rev = '65a42de1148dbed2297e922b5dbc7d9b70c36578'; file = 'mmproj-GLM-OCR-Q8_0.gguf'; sha = '9c4b58e33e316ed142eb5dcb41abec3844d3e6e5dc361ffb782c3fa9d175141f' }
)
$page = @{ url = 'https://raw.githubusercontent.com/fmadore/Kalamu/main/samples/bulletin-page.jpg' }

function Get-Checked([string]$url, [string]$path, [string]$sha) {
	if (Test-Path $path) {
		if (-not $sha -or (Get-FileHash -Algorithm SHA256 $path).Hash -eq $sha) { return }
	}
	Write-Host "Downloading $(Split-Path $path -Leaf)"
	& curl.exe -fsSL --retry 5 --retry-delay 10 -o $path $url
	if ($LASTEXITCODE -ne 0) { throw "Could not download $url" }
	if ($sha) {
		$actual = (Get-FileHash -Algorithm SHA256 $path).Hash
		if ($actual -ne $sha) { Remove-Item $path; throw "$path has SHA-256 $actual, expected $sha" }
	}
}

$triple = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
$bin = Join-Path $Work 'bin'
$modelDir = Join-Path $Work 'models'
$out = Join-Path $Work 'out'
New-Item -ItemType Directory -Force $bin, $modelDir, $out | Out-Null

foreach ($kind in 'llama', 'vulkan') {
	$name = if ($kind -eq 'llama') { "llama-$tag-$triple.zip" } else { "llama-vulkan-$tag-$triple.zip" }
	$zip = Join-Path $Work $name
	Get-Checked "https://github.com/fmadore/Kalamu/releases/download/$release/$name" $zip $archives[$triple][$kind]
	& "$env:SystemRoot\System32\tar.exe" -xf $zip -C $bin
}
foreach ($m in $modelFiles) {
	Get-Checked "https://huggingface.co/$($m.repo)/resolve/$($m.rev)/$($m.file)" (Join-Path $modelDir $m.file) $m.sha
}
$pagePath = Join-Path $Work 'bulletin-page.jpg'
Get-Checked $page.url $pagePath ''

# As crates/offline/src/cpu.rs: half the logical processors on x64, all but one on ARM64, 1 to 8.
$logical = [Environment]::ProcessorCount
$threads = if ($triple -like 'x86_64*') { [math]::Floor($logical / 2) } else { $logical - 1 }
$threads = [math]::Max(1, [math]::Min(8, $threads))

$cpuName = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name
Write-Host ""
Write-Host "Processor: $cpuName ($logical logical, $threads threads)"
Get-CimInstance Win32_VideoController | ForEach-Object { Write-Host "Graphics: $($_.Name), driver $($_.DriverVersion)" }

function Get-Devices([string]$program) {
	$listed = & $program --list-devices 2>&1 | Out-String
	Write-Host $listed.Trim()
	$listed -split "`n" | ForEach-Object {
		if ($_ -match '^\s+(\S+): (.+) \((\d+) MiB') { $Matches[1] }
	}
}

function Invoke-Page([string]$label, [string]$program, [string]$device, [string]$name, [string]$model, [string]$encoder, [string]$prompt) {
	$arguments = @('-m', (Join-Path $modelDir $model), '--mmproj', (Join-Path $modelDir $encoder),
		'--image', $pagePath, '-p', "<__media__>$prompt`n", '--jinja', '--temp', '0',
		'-n', '6144', '-c', '10240', '-b', '4096', '-ub', '4096', '-t', "$threads", '-ngl', '99',
		'--no-warmup')
	if ($device) { $arguments += @('--device', $device) }
	# The label names the device (`vulkan:Vulkan0`): a colon, among others, is refused in a
	# Windows file name.
	$file = ($label -replace '[<>:"/\\|?*]', '-') + "-$name"
	$answer = Join-Path $out "$file.txt"
	$log = Join-Path $out "$file.log"
	$watch = [Diagnostics.Stopwatch]::StartNew()
	$p = Start-Process -FilePath $program -ArgumentList ($arguments | ForEach-Object { '"' + ($_ -replace '"', '\"') + '"' }) `
		-NoNewWindow -Wait -PassThru -RedirectStandardOutput $answer -RedirectStandardError $log
	$watch.Stop()
	$text = Get-Content -Raw -Encoding UTF8 $answer
	$title = [bool]($text -match '(?i)bulletin')
	$blocks = ([regex]::Matches($text, '(?m)^!\[[a-z_+]+\]\(\d+, ?\d+, ?\d+, ?\d+\)')).Count
	$encoding = (Select-String -Path $log -Pattern 'encoding done in (\d+) ms' | Select-Object -Last 1).Matches.Groups[1].Value
	[pscustomobject]@{
		Build = $label; Model = $name; Seconds = [math]::Round($watch.Elapsed.TotalSeconds, 1)
		ImageSeconds = if ($encoding) { [math]::Round([int]$encoding / 1000, 1) } else { $null }
		Exit = $p.ExitCode; TitleRead = $title; Blocks = $blocks
	}
}

function Invoke-Both([string]$label, [string]$program, [string]$device) {
	if ($Models -contains 'lightonocr') {
		Invoke-Page $label $program $device 'lightonocr' 'LightOnOCR-3-1B.Q8_0.gguf' 'LightOnOCR-3-1B.mmproj-q8_0.gguf' 'grounding'
	}
	if ($Models -contains 'glm-ocr') {
		Invoke-Page $label $program $device 'glm-ocr' 'GLM-OCR-Q8_0.gguf' 'mmproj-GLM-OCR-Q8_0.gguf' 'Text Recognition:'
	}
}

$results = @()
$results += Invoke-Both 'cpu' (Join-Path $bin 'llama-mtmd-cli.exe') ''
if (-not $ProcessorOnly) {
	Write-Host ""
	Write-Host "Vulkan build:"
	$vulkan = Join-Path $bin 'llama-mtmd-cli-vulkan.exe'
	foreach ($device in (Get-Devices $vulkan)) {
		$results += Invoke-Both "vulkan:$device" $vulkan $device
	}
}
if ($OpenClDir -and -not $ProcessorOnly) {
	Write-Host ""
	Write-Host "OpenCL test build:"
	$opencl = Join-Path $OpenClDir 'llama-mtmd-cli-opencl.exe'
	foreach ($device in (Get-Devices $opencl)) {
		$results += Invoke-Both "opencl:$device" $opencl $device
	}
}
Write-Host ""
$results | Format-Table -AutoSize
Write-Host "Answers and logs: $out"
