fd -HI --type f -E .git -E vendor '.*\.php' | sed -u -E 's/\\/\//g' | \
  xargs -n1 -P4 php -l 2>&1 1>/dev/null | \
  grep --line-buffered -E .
