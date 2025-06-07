$ErrorActionPreference = 'Stop'

function Ensure-Tool {
    param(
        [string]$Name,
        [string]$Command
    )
    if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) {
        Write-Host "ERROR: $Name ('$Command') not found on PATH." -ForegroundColor Red
        exit 1
    }
}

# Basic tooling
Ensure-Tool 'Rust' 'cargo'
Ensure-Tool 'Node.js' 'node'
Ensure-Tool 'pnpm' 'pnpm'

# Load Visual Studio environment if necessary
if (-not $env:VCINSTALLDIR) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (Test-Path $vswhere) {
        $vsPath = & $vswhere -latest -products * -requires Microsoft.Component.MSBuild -property installationPath
        if ($vsPath) {
            $vsDevCmd = Join-Path $vsPath 'Common7\Tools\VsDevCmd.bat'
            if (Test-Path $vsDevCmd) {
                Write-Host "Loading Visual Studio environment from $vsDevCmd"
                cmd /c "`"$vsDevCmd`" -arch=amd64 -host_arch=amd64 && set" | ForEach-Object {
                    if ($_ -match '^(.*?)=(.*)$') { Set-Item -Path Env:\$($matches[1]) -Value $matches[2] }
                }
            }
        }
    }
    if (-not $env:VCINSTALLDIR) {
        Write-Host 'ERROR: Visual Studio Build Tools not found. Install "Desktop development with C++" and restart.' -ForegroundColor Red
        exit 1
    }
}

# Install front-end dependencies required by Tauri
if (Test-Path './app/package.json') {
    Write-Host 'Installing front-end dependencies…'
    pushd ./app
    if (-not (Test-Path node_modules)) { pnpm install } else { pnpm install --frozen-lockfile }
    popd
}

# Sanity check before building
cargo check
