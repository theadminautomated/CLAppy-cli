cargo tauri build --windows --release
$msi = Get-ChildItem target\release\bundle\msi\*.msi
Copy-Item -Recurse share\examples target\release\bundle\msi\
& makensis /DMSI=$msi installer.nsi
& pkgforge pack installer\nsis\clappy_installer.exe -o dist\clappy.exe --silent
