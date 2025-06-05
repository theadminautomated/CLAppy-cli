cargo tauri build --windows --release
$msi = Get-ChildItem target\release\bundle\msi\*.msi
& makensis /DMSI=$msi installer.nsi
& pkgforge pack installer\nsis\openwarp_installer.exe -o dist\openwarp.exe
