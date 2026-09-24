# Solyn installer for Windows.
# Usage: irm https://raw.githubusercontent.com/Keeferf/Solyn/main/install.ps1 | iex
$ErrorActionPreference = "Stop"

$repo = "Keeferf/Solyn"
$release = Invoke-RestMethod `
  -Uri "https://api.github.com/repos/$repo/releases/latest" `
  -Headers @{ "User-Agent" = "solyn-install" }

$asset = $release.assets | Where-Object { $_.name -like "*x64_en-US.msi" } | Select-Object -First 1
if (-not $asset) {
  $asset = $release.assets | Where-Object { $_.name -like "*x64-setup.exe" } | Select-Object -First 1
}
if (-not $asset) {
  throw "No Solyn installer found in the latest release."
}

$out = Join-Path $env:TEMP $asset.name
Write-Host "Downloading $($asset.browser_download_url)"
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $out

if ($out.EndsWith(".msi")) {
  Start-Process msiexec.exe -ArgumentList "/i `"$out`"" -Wait
} else {
  Start-Process $out -ArgumentList "/S" -Wait
}

Write-Host "Done. Solyn installs Ollama on first launch."
