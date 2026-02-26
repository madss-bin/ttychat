$LOGO_FILE = "assets\logo.txt"

$C_BLUE = "`e[38;5;39m"
$C_PURPLE = "`e[38;5;135m"
$C_PINK = "`e[38;5;213m"
$C_GREEN = "`e[38;5;82m"
$C_GREY = "`e[38;5;240m"
$YELLOW = "`e[1;33m"
$NC = "`e[0m"

function Show-Logo {
    if (Test-Path $LOGO_FILE) {
        Write-Host ""
        Write-Host "$C_BLUE"
        Get-Content $LOGO_FILE
        Write-Host "$NC"
        Write-Host ""
    }
}

function Run-Step {
    param(
        [string]$Description,
        [string[]]$Commands
    )

    Write-Host "$C_PURPLE:: $C_BLUE$Description$NC"
    Write-Host ""
    Write-Host ""
    Write-Host ""
    
    $totalSteps = $Commands.Count

    for ($i = 0; $i -lt $totalSteps; $i++) {
        $cmd = $Commands[$i]
        $stepNum = $i + 1
        $percent = [math]::Round(($stepNum * 100) / $totalSteps)
        
        $width = 40
        $filled = [math]::Floor(($percent * $width) / 100)
        $empty = $width - $filled
        $bar = "━" * $filled
        $space = "━" * $empty
        Write-Host -NoNewline "`e[3A"
        Write-Host -NoNewline "`r`e[K"
        Write-Host "$C_GREY> $cmd$NC"
        Write-Host -NoNewline "`r`e[K"
        Write-Host "$C_GREEN▕$C_PINK$bar$C_GREY$space$C_GREEN▏ $C_PINK$percent%$NC"
        Invoke-Expression "$cmd 2>&1" | ForEach-Object {
            $line = $_.ToString()
            if ($line.Length -gt 70) { $line = $line.Substring(0, 70) }
            Write-Host -NoNewline "`r`e[K$C_GREY$line$NC"
        }

        if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne $null) {
            Write-Host "`n$C_PINK Command failed: $cmd$NC"
            exit 1
        }
        
        Write-Host -NoNewline "`r`e[K"
    }
    Write-Host ""
}

Show-Logo

Write-Host "Detected OS: ${YELLOW}Windows${NC}"

if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "$YELLOW`Cargo not found. Please install Rust from https://rustup.rs$NC"
    exit 1
}

Run-Step "Building ttychat" @("cargo build --release")

$installDir = "$HOME\AppData\Local\ttychat\bin"
if (!(Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

Run-Step "Installing to System" @(
    "Copy-Item target\release\ttychat.exe $installDir\ttychat.exe -Force"
)

Write-Host ""
Write-Host "$C_GREEN✓ ttychat installed successfully!$NC"
Write-Host "The binary is located at: $C_BLUE$installDir\ttychat.exe$NC"
Write-Host "To run ttychat from anywhere, add this folder to your PATH."
