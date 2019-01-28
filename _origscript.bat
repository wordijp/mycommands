@rem ÉRÉ}ÉìÉhçÏê¨) mklink new.bat _origscript.bat

@if exist %~dpn0.sh (
  @bash %~dpn0.sh %*
) else if exist %~dpn0.py (
  @python -u %~dpn0.py %*
) else if exist %~dpn0.rb (
  @ruby -r%~dp0_preproc.rb --disable=gems %~dpn0.rb %*
) else (
  @echo script not found: %~n0
)
