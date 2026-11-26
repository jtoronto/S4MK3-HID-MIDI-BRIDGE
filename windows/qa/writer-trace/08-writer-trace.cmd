@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Start-WriterTrace.ps1" %*
set "result=%ERRORLEVEL%"
echo.
echo Writer trace launcher exit code: %result%
pause
exit /b %result%
