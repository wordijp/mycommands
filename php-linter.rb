# 複数のLinterでチェック

require 'parallel'
require 'open3'

PHPMD_DEFAULT = 'text codesize,design,unusedcode'

def main(argv)
  cmd = argv.shift
  case cmd
  when 'php-l'
    exec "#{__dir__}/internal/php-lint-run"
  when 'phan'
    exec "#{__dir__}/internal/phan-run"
  when 'phpmd'
    argv = argv.empty? ? PHPMD_DEFAULT : argv.join(' ')
    exec "#{__dir__}/internal/phpmd-run #{argv}"
  when 'multi'
    linterMulti argv.join(' ')
  when 'help'
    usage
  when /.+/
    usage
  else
    exec "#{__dir__}/internal/php-lint-run"
  end
end

def usage
  puts <<-EOS
usage) #{File.basename(__FILE__)} [<mode> | help]

mode:
  php-l   linter by php-lint [default]
  phan    linter by phan
  phpmd   linter by phpmd
  multi   multiple linter(phan, phpmd, and php-l)

help: this message
  EOS
end

def linterMulti(phpmdArgv)
  # php -l
  # NOTE: これは早いので先に済ませる
  lines = exec "#{__dir__}/internal/php-lint-run"
  if lines > 0
    exit 0
  end

  # NOTE: 以降は時間がかかるので並列実行
  
  argv = phpmdArgv.empty? ? PHPMD_DEFAULT : phpmdArgv.join(' ')
  # phpmd
  cmd_phpmd = "#{__dir__}/internal/phpmd-run #{argv}"
  # phan
  cmd_phan = "#{__dir__}/internal/phan-run"

  msgsLst = Parallel.map([cmd_phpmd, cmd_phan], in_threads: 2) do |cmd|
    `#{cmd}`
  end
  msgsLst.each do |_msgs|
    unless _msgs.empty?
      print _msgs
      exit 0
    end
  end
end

# @return 行数
def exec(cmd)
  lines = 0
  Open3.popen3(cmd) {|_, o|
    o.each_line do |line|
      puts line
      lines += 1
    end
  }
  lines
end

main ARGV
