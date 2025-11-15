$priv = openssl genpkey -algorithm ed25519
Write-Host "private key:" -ForegroundColor Yellow
$priv
Write-Host "public key:" -ForegroundColor Yellow
$priv | openssl pkey -pubout
