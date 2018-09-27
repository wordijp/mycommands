#!/bin/bash
# PHPファイル一覧をphpmd Linterでチェックする

fd -HI --type f -E .git -E vendor '.*\.php' | sed -E 's/\\/\//g' | \
	paste -s -d, | \
	xargs -I'{}' phpmd {} $*
