#!/bin/bash
# PHPファイル一式をPHP内蔵Linter(php -l)でチェックする

fd -HI --type f -E .git -E vendor '.*\.php' | sed -E 's/\\/\//g' | \
	xargs -I'{}' -n1 php -l {} 2>&1 1>/dev/null | \
	grep -v -E '^No syntax errors'
