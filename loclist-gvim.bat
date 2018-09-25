@echo off
@rem QuickFixロケーションリストを選択し、gvimで開く
@rem -r: 前回の履歴から開く

set g_TEMP_FILE=%TEMP%\vim_loclist.tmp

if "%1" equ "-r"       goto resume
if "%1" equ "--resume" goto resume
if "%1" neq ""         goto filetype

echo usage) %0 ^<filetype^>
exit /B 0

:resume
@rem 履歴から復元
set g_CAT_CMD=cat %g_TEMP_FILE%
goto run

:filetype
@rem エラーメッセージをfiletypeでパースする
set g_CAT_CMD=%~dp0internal\vim-loclist %1 ^| tee %g_TEMP_FILE%
goto run

:run
%g_CAT_CMD% | fzf --exit-0 --reverse | %~dp0internal\vim-arg | %~dp0internal\xexec --async gvim --remote-silent
