#!/bin/bash
# QuickFixロケーションリストをVim用引数へ変換する
# result) +行 path/to/file

cat - | \
	sed -E 's/([^|]+)\|([0-9]+)( col [0-9]+)?\|.+$/+\2 \1/'
