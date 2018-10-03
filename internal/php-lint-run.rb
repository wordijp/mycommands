require 'open3'

# original)
#fd -HI --type f -E .git -E vendor '.*\.php' | sed -u -E 's/\\/\//g' | \
#  xargs -I'{}' -n1 php -l {} 2>&1 1>/dev/null | \
#  grep --line-buffered -v -E '^No syntax errors'

Open3.popen3('fd -HI --type f -E .git -E vendor ".*\.php$" | sed -E "s/\\\\\\\\/\\\\//g"') {|i, o|
  o.each do |line|
    Open3.popen3("php -l #{line}") {|_, _, e2|
      e2.each do |line2|
        print line2 unless line2.match(/^No syntax errors/)
      end
    }
  end
}
