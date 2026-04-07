$ErrorActionPreference = 'Stop'

# Install Sunshine silently
$installer = Join-Path $PSScriptRoot 'Sunshine-Windows-AMD64-installer.exe'
if (!(Test-Path $installer)) {
  $installer = Get-ChildItem $PSScriptRoot -Filter '*Sunshine*installer*.exe' | Select-Object -First 1 -ExpandProperty FullName
}
if (!(Test-Path $installer)) { throw 'Sunshine installer not found in this folder.' }

Write-Host "Installing Sunshine from $installer"
Start-Process -FilePath $installer -ArgumentList '/S' -Wait

# Open firewall commonly used by Sunshine (TCP 47984/47989/47990/48010, UDP 47998-48000/8000)
$portsTcp = @(47984,47989,47990,48010)
$portsUdp = @(47998,47999,48000,8000)
foreach ($p in $portsTcp) {
  netsh advfirewall firewall add rule name="Sunshine TCP $p" dir=in action=allow protocol=TCP localport=$p | Out-Null
}
foreach ($p in $portsUdp) {
  netsh advfirewall firewall add rule name="Sunshine UDP $p" dir=in action=allow protocol=UDP localport=$p | Out-Null
}

# Start Sunshine service if present
$svc = Get-Service -Name 'SunshineService' -ErrorAction SilentlyContinue
if ($svc) {
  Set-Service -Name SunshineService -StartupType Automatic
  Start-Service -Name SunshineService
}

Write-Host "Done. Open Sunshine UI in Windows, set PIN, then pair from Moonlight on host."
