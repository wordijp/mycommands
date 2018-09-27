# 複数のLinterでチェック

require 'parallel'

PHPMD_DEFAULT = 'text codesize,design,unusedcode'

def main(argv)
  cmd = argv.shift
  case cmd
  when 'php-l'
    print `#{__dir__}/internal/php-lint-run`
  when 'phan'
    print `#{__dir__}/internal/phan-run`
  when 'phpmd'
    argv = argv.empty? ? PHPMD_DEFAULT : argv.join(' ')
    print `#{__dir__}/internal/phpmd-run #{argv}`
  when 'multi'
    linterMulti argv.join(' ')
  when 'help'
    usage
  when /.+/
    usage
  else
    print `#{__dir__}/internal/php-lint-run`
  end
end

def usage
  puts <<-EOS
usage) #{File.basename(__FILE__)} [<mode> | help]

mode:
  php-l   linter by php-lint [default]
  phan    linter by phan
  phpmd   linter by phpmd
  multi   multiple linter(phan, php-l)

help: this message
  EOS
end

def linterMulti(phpmdArgv)
  # php -l
  # NOTE: これは早いので先に済ませる
  msgs = `#{__dir__}/internal/php-lint-run`
  unless msgs.empty?
    print msgs
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

main ARGV
