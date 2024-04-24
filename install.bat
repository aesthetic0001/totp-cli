@echo off
setlocal

:: Get the latest release from the GitHub API
for /f "delims=" %%i in ('curl -s https://api.github.com/repos/aesthetic0001/totp-cli/releases/latest') do set "latest_release=%%i"

:: Get the download URL for the asset using PowerShell
for /f "delims=" %%i in ('powershell -Command "($env:latest_release | ConvertFrom-Json).assets | Where-Object name -eq 'totp-windows.exe' | Select-Object -ExpandProperty browser_download_url"') do set "download_url=%%i"

:: Create a directory for the tool
if not exist "%USERPROFILE%\totp-cli\" mkdir "%USERPROFILE%\totp-cli\"

:: Download the asset and rename it to totp
curl -L -o "%USERPROFILE%\totp-cli\totp" %download_url%

:: Add the directory to the system PATH
setx path "%path%;%USERPROFILE%\totp-cli\"

:: Check if the asset was installed successfully
if exist "%USERPROFILE%\totp-cli\totp" (
    echo totp-cli was installed successfully
) else (
    echo totp-cli could not be installed
)

endlocal