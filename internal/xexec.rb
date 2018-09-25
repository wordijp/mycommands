# 標準入力とコマンドライン引数の両方を使い実行する
line = STDIN.gets
Kernel.exit true if line.nil?
line.chomp!

async = false

args = ARGV.map { |arg|
  if arg == '--async'
    async = true
    ''
  else
    "#{arg.chomp}"
  end
}
  .select { |x| x != '' }
  .join(' ')

if async
  # 非同期
  Process.detach(spawn("#{args} #{line}"))
else
  # 同期
  `#{args} #{line}`
end
