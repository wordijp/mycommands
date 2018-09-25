@rem コマンド作成) mklink new.bat _orig.bat

@if exist %~dpn0.sh (
  @bash %~dpn0.sh %*
) else if exist %~dpn0.rb (
  @ruby %~dpn0.rb %*
) else (
  @echo script not found: %~n0
)
