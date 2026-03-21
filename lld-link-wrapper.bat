@echo off
setlocal enabledelayedexpansion

set REAL_LLDLINK=C:\msys64\clang64\bin\lld-link.exe
set ARGS=

:loop
if "%~1"=="" goto done
set ARG=%~1

rem Convert /DEFAULTLIB:*libgcc.lib and /DEFAULTLIB:*libgcc_eh.lib to /WHOLEARCHIVE:
set TESTARG=%ARG%
if not "!TESTARG:/DEFAULTLIB:=!"=="!TESTARG!" (
  set INNER=!TESTARG:/DEFAULTLIB:=!
  echo !INNER! | findstr /i "libgcc\.lib libgcc_eh\.lib" >/dev/null 2>&1
  if not errorlevel 1 (
    set ARGS=!ARGS! "/WHOLEARCHIVE:!INNER!"
    shift
    goto loop
  )
)
set ARGS=!ARGS! "%ARG%"
shift
goto loop

:done
%REAL_LLDLINK% %ARGS%
