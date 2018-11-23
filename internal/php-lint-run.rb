require 'parallel'
require 'open3'

# original)
#fd -HI --type f -E .git -E vendor '.*\.php' | sed -u -E 's/\\/\//g' | \
#  xargs -I'{}' -n1 php -l {} 2>&1 1>/dev/null | \
#  grep --line-buffered -v -E '^No syntax errors'


def main
  cmd = 'fd -HI --type f -E .git -E vendor ".*\.php$" | sed -E "s/\\\\\\\\/\\\\//g"'
  files = `#{cmd}`.lines.map {|file| file.chomp}

  threads = Parallel.processor_count
  Parallel.each_with_index(files, in_threads: threads) do |file, i|
    Open3.popen3("php -l #{file}") do |_, o, e|
      # check have error
      error = true
      o.each do |line|
        if line.index('No syntax errors') != nil
          error = false
          break
        end
      end

      msg = e.first
      stablePrint(i, error, msg)
    end
  end
end


$msgs = {}
$cur_i = 0

# 順番を保ったまま出力
def stablePrint(i, error, msg)
  # メッセージ更新
  $msgs[i] = {
    error: error,
    msg: msg
  }

  # メッセージ表示
  while $msgs.has_key?($cur_i)
    msg_ = $msgs[$cur_i]
    if msg_[:error]
      print msg_[:msg]
    end

    $msgs.delete($cur_i)
    $cur_i += 1
  end
end

main
