#!/bin/bash
# PHPファイル一覧をphpmd Linterでチェックする

fd -HI --type f -E .git -E vendor '.*\.php' | sed -E 's/\\/\//g' | \
	xargs -I'{}' phpmd {} $*
