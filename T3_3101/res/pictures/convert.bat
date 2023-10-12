@echo off
echo "Konvertiere PDF Dateien"
for %%f in (*.svg) do (
    echo Convert %%f
    "C:\Program Files\Inkscape\bin\inkscape.exe" -f "%%~nf.svg" -A "%%~nf.pdf"
)