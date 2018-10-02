@rem コマンド作成) mklink new.bat _origscript.bat

@if exist %~dpn0.sh (
  @bash %~dpn0.sh %*
) else if exist %~dpn0.rb (
  @ruby -r%~dp0_preproc.rb %~dpn0.rb %*
) else (
  @echo script not found: %~n0
)
