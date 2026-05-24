$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject "CN=Photopipeline" `
  -FriendlyName "Photopipeline Code Signing" `
  -KeyUsage DigitalSignature `
  -KeyAlgorithm RSA `
  -KeyLength 4096 `
  -NotAfter (Get-Date).AddYears(5) `
  -CertStoreLocation "Cert:\CurrentUser\My"

$pwd = ConvertTo-SecureString -String "photopipeline-dev" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath "photopipeline-codesign.pfx" -Password $pwd

Write-Host "Certificate thumbprint: $($cert.Thumbprint)"
Write-Host "PFX saved to: photopipeline-codesign.pfx"
