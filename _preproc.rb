# rubyスクリプトの事前処理

STDOUT.sync=true

Signal.trap(:INT) do
  exit(2)
end
