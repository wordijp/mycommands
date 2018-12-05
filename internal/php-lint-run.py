# -*- coding: utf-8 -*-

# original)
#fd -HI --type f -E .git -E vendor '.*\.php' | sed -u -E 's/\\/\//g' | \
#  xargs -I'{}' -n1 php -l {} 2>&1 1>/dev/null | \
#  grep --line-buffered -v -E '^No syntax errors'

import os
import re
import subprocess

from concurrent.futures import ThreadPoolExecutor
from threading import Lock



def main():
    cmd = 'fd -HI --type f -E .git -E vendor ".*\.php$" | sed -E "s/\\\\\\\\/\\\\//g"'
    files, _ = run(cmd)

    with ThreadPoolExecutor(max_workers=os.cpu_count()) as executer:
        executer.map(doLint, zip(range(len(files)), files))


lock = Lock()

# linter処理
def doLint(tpl):
    i, file = tpl

    o, e = run('php -l {}'.format(file))

    error = not any(map(lambda line: re.search('^No syntax errors', line), o))
    msg = e[0] if len(e) > 0 else ''

    global lock
    with lock:
        stablePrint(i, error, msg)

# 外部コマンドを実行し、出力を戻り値で返す
def run(cmd):
    outs = []
    errs = []

    proc = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    while True:
        out = proc.stdout.readline()
        err = proc.stderr.readline()
        if not out and not err:
            break

        outs.append(out.decode('utf-8').rstrip())
        errs.append(err.decode('utf-8').rstrip())

    return outs, errs


msgs = {}
cur_i = 0
# 順番を保ったまま出力
def stablePrint(i, error, msg):
    global msgs
    global cur_i

    # メッセージ更新
    msgs[i] = {
        'error': error,
        'msg': msg
    }

    # メッセージ表示
    while cur_i in msgs:
        msg = msgs[cur_i]
        if msg['error']:
            print(msg['msg'])

        del msgs[cur_i]
        cur_i += 1


if __name__ == '__main__':
    main()
