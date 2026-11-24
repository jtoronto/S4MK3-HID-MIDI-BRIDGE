@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0JOGT-Trial.ps1"
if errorlevel 1 pause
