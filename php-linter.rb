# 複数のLinterでチェック

def main(argv)
  cmd = argv.shift
  case cmd
  when 'php-l'
    print `#{__dir__}/internal/php-lint-run`
  when 'phan'
    print `#{__dir__}/internal/phan-run`
  when 'phpmd'
    print `#{__dir__}/internal/phpmd-run #{argv.join(' ')}`
  when 'multi'
    linterMulti
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

def linterMulti
  # php -l
  msgs = `#{__dir__}/internal/php-lint-run`
  unless msgs.empty?
    print msgs
    exit 0
  end

  # phan
  msgs = `#{__dir__}/internal/phan-run`
  unless msgs.empty?
    print msgs
    exit 0
  end
end

main ARGV
