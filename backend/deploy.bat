@echo off
rem Build the backend, copy the exe into an "active" folder and (re)start it there.
rem The running copy is separate from target\, so you can keep rebuilding/running a dev instance.
rem
rem usage:  deploy.bat [port] [active_dir]
rem   port        default 8900
rem   active_dir  default .\active   (next to this script)
rem
rem examples:
rem   deploy.bat                  live instance on 8900
rem   deploy.bat 8901 staging     second instance on 8901 in its own folder
rem
rem The server runs under a small supervisor loop (this script again, with ":supervise"):
rem if the process exits or crashes, it's restarted after 3 s. A redeploy stops it on purpose
rem through a ".stop" file next to the log, so the supervisor doesn't restart the old exe.

rem supervisor mode: started by this script below, not meant to be called by hand
if "%~1"==":supervise" goto :supervise

setlocal

set "PORT=%~1"
if "%PORT%"=="" set "PORT=8900"

set "SCRIPT_DIR=%~dp0"
set "ACTIVE_DIR=%~2"
if "%ACTIVE_DIR%"=="" set "ACTIVE_DIR=%SCRIPT_DIR%active"

rem own exe name per port, so stopping this instance never hits the dev one (or another port)
set "BIN_NAME=catastrophe-%PORT%.exe"
set "SRC_BIN=%SCRIPT_DIR%target\release\catastrophe-backend.exe"
set "DST_BIN=%ACTIVE_DIR%\%BIN_NAME%"
set "LOG_FILE=%ACTIVE_DIR%\catastrophe-%PORT%.log"
set "STOP_FILE=%ACTIVE_DIR%\catastrophe-%PORT%.stop"

rem --- validate port: digits only, 1..65535 ---
echo %PORT%| findstr /r "^[0-9][0-9]*$" >nul || goto :bad_port
if %PORT% LSS 1 goto :bad_port
if %PORT% GTR 65535 goto :bad_port

echo ==^> building release
pushd "%SCRIPT_DIR%"
cargo build --release
if errorlevel 1 (
    popd
    echo error: build failed  restarting old build 1>&2
)
popd

if not exist "%SRC_BIN%" (
    echo error: %SRC_BIN% not found - did the build succeed? 1>&2
    exit /b 1
)

if not exist "%ACTIVE_DIR%" mkdir "%ACTIVE_DIR%"

echo ==^> stopping old instance on port %PORT% (if any)
rem tell the supervisor this exit is on purpose, otherwise it would restart the old exe right away
tasklist /FI "IMAGENAME eq %BIN_NAME%" | find /i "%BIN_NAME%" >nul && (echo stop> "%STOP_FILE%")
taskkill /F /IM "%BIN_NAME%" >nul 2>&1
rem wait up to ~5 s until it's gone, so the exe file is unlocked again
for /l %%i in (1,1,5) do (
    tasklist /FI "IMAGENAME eq %BIN_NAME%" | find /i "%BIN_NAME%" >nul || goto :copy
    ping -n 2 127.0.0.1 >nul
)

:copy
rem the supervisor deletes the stop file when it quits. If none was running (e.g. an instance
rem started by an older version of this script), nobody does - so wait a moment, then remove it
for /l %%i in (1,1,3) do (
    if not exist "%STOP_FILE%" goto :stopped
    ping -n 2 127.0.0.1 >nul
)
del "%STOP_FILE%" >nul 2>&1
:stopped

echo ==^> copying to %DST_BIN%
copy /Y "%SRC_BIN%" "%DST_BIN%" >nul
if errorlevel 1 (
    echo error: could not copy the exe - is the old instance still running? 1>&2
    exit /b 1
)

rem port must be free now - otherwise something else (e.g. "cargo run") already uses it
netstat -ano -p TCP | findstr /r /c:"0\.0\.0\.0:%PORT%  *0\.0\.0\.0:0 " >nul
if not errorlevel 1 (
    echo error: port %PORT% is already used by another program: 1>&2
    netstat -ano -p TCP | findstr /r /c:"0\.0\.0\.0:%PORT%  *0\.0\.0\.0:0 " 1>&2
    echo the last number is the process id - look it up with: tasklist /FI "PID eq <id>" 1>&2
    exit /b 1
)

echo ==^> starting on port %PORT% (log: %LOG_FILE%)
rem own minimized window, so it keeps running after this terminal is closed.
rem Note: don't pipe this script's output (deploy.bat | ...) - the server inherits the pipe and
rem the pipe then only closes when the server stops. Running it normally is fine.
rem The window runs the supervisor loop at the end of this file, which (re)starts the server.
start "catastrophe-%PORT%" /MIN /D "%ACTIVE_DIR%" cmd /c ""%~f0" :supervise "%DST_BIN%" %PORT% "%LOG_FILE%" "%STOP_FILE%""

rem wait until the port is actually listening. The first start of a freshly copied exe can take
rem a few seconds (Windows Defender scans it), so poll for up to ~20 s instead of checking once.
for /l %%i in (1,1,20) do (
    rem listening socket = local 0.0.0.0:PORT with remote 0.0.0.0:0
    rem (the state column is translated, e.g. "ABHOEREN" on German Windows, so don't match on it)
    netstat -ano -p TCP | findstr /r /c:"0\.0\.0\.0:%PORT%  *0\.0\.0\.0:0 " >nul && goto :running
    ping -n 2 127.0.0.1 >nul
)
echo error: server is not listening on port %PORT% after 20 s, log: 1>&2
type "%LOG_FILE%" 1>&2
exit /b 1

:running
echo ==^> running on http://localhost:%PORT%/status
exit /b 0

:bad_port
echo error: port must be a number between 1 and 65535, got "%PORT%" 1>&2
exit /b 1


rem ---------------------------------------------------------------------------------------------
rem supervisor: runs inside the minimized window, restarts the server whenever it exits
rem args: :supervise <exe> <port> <log file> <stop file>
:supervise
set "SV_BIN=%~2"
set "SV_PORT=%~3"
set "SV_LOG=%~4"
set "SV_STOP=%~5"

:supervise_loop
>> "%SV_LOG%" echo [%date% %time%] starting server on port %SV_PORT%
"%SV_BIN%" %SV_PORT% >> "%SV_LOG%" 2>&1
set "SV_EXIT=%errorlevel%"

rem stopped on purpose by a redeploy -> don't restart
if exist "%SV_STOP%" (
    del "%SV_STOP%" >nul 2>&1
    >> "%SV_LOG%" echo [%date% %time%] server stopped for redeploy
    exit /b 0
)

>> "%SV_LOG%" echo [%date% %time%] server exited with code %SV_EXIT%, restarting in 3 s
ping -n 4 127.0.0.1 >nul
goto :supervise_loop
