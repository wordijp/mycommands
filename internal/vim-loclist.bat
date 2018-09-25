@echo off
@rem エラー一覧をQuickFixロケーションリストへ変換する

if "%1" == "" (
  echo "usage) vim-loclist <filetype>"
  exit /B 0
)

vim - -es +"source $VIM/vimrc" +"set nonumber" +"set filetype=%1" +":lgetbuffer" +":lopen" +:%%p +:q! +:q! | tail +2 | grep -v -E "^\|\|\s*$"
