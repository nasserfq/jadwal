# جدول in the Microsoft Store

`make-msix.ps1` turns the files of the Windows download into
`jadwal-<version>-windows-x64-store.msix`. The release workflow builds it on every push to
`release` and keeps it as the **microsoft-store** workflow artifact (at the bottom of the run's page,
visible when signed in to GitHub); it is not attached to the GitHub release, so the website
downloads stay as they are.

The package identity in `AppxManifest.xml.in` is the one Partner Center reserved for
جدول بيانات (Product management › Product identity). The Store signs the package when it is
uploaded.

## What changes in a Store copy

A copy installed from the Store runs from a `WindowsApps` folder (`craft_update::store`). It never
checks for or downloads updates, and Help › Check for Updates opens the Store's updates page. The
same `Jadwal.exe` is in the website zip, which keeps its own updater.

The package has one application. A second one hidden from Start (a `jadwal-cli.exe` alias) is refused
by the Store as a "headless app" without a waiver from Microsoft, so MCP users take the command line
from the GitHub download.

## Trying the package before uploading

On a Windows PC with the Windows SDK, as administrator:

```powershell
$cert = New-SelfSignedCertificate -Type Custom -Subject "CN=AF2BB125-FF7A-4337-932D-42917A905572" `
  -KeyUsage DigitalSignature -CertStoreLocation Cert:\CurrentUser\My `
  -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
Export-Certificate -Cert $cert -FilePath test.cer
Import-Certificate -FilePath test.cer -CertStoreLocation Cert:\LocalMachine\TrustedPeople
Copy-Item jadwal-*-store.msix test.msix
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe" sign /fd SHA256 /sha1 $cert.Thumbprint test.msix
Add-AppxPackage test.msix
```

Then check: open an .xlsx from Explorer, edit, save, print. Remove it with `Get-AppxPackage *2255423B0420* | Remove-AppxPackage`.

## Uploading

Partner Center › جدول بيانات › Start submission (or Update) › Packages: upload the unsigned
`.msix` from the workflow artifact. Each new upload needs a higher version, which the workflow's run
number gives. `runFullTrust` gives a warning on upload: explain it under Submission options ›
Restricted capabilities ("a desktop spreadsheet app that opens and saves the user's files").
