$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$target = Join-Path $projectRoot '.env'
if (Test-Path -LiteralPath $target) { Write-Output 'Existing .env preserved. Compare with .env.example.'; exit 0 }
function New-LocalSecret {
    $bytes = New-Object byte[] 32
    $generator = [Security.Cryptography.RandomNumberGenerator]::Create()
    $generator.GetBytes($bytes)
    $generator.Dispose()
    return ([BitConverter]::ToString($bytes)).Replace('-', '').ToLowerInvariant()
}
$config = @("POSTGRES_PASSWORD=$(New-LocalSecret)", "JWT_SECRET=$(New-LocalSecret)", "AI_SERVICE_TOKEN=$(New-LocalSecret)")
[IO.File]::WriteAllLines($target, $config)
Write-Output 'Created local .env with random secrets. Start Docker and run docker compose up --build.'
