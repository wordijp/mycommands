ncpu=`cat /proc/cpuinfo | grep processor | wc -l | tr -d '\n'`

fd -HI --type f -E .git -E vendor '.*\.php' | sed -u -E 's/\\/\//g' | \
  xargs -n1 -P$ncpu php -l 2>&1 1>/dev/null | \
  grep --line-buffered -E .
