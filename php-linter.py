# -*- coding: utf-8 -*-

import os
import sys
import subprocess
import re

from concurrent.futures import ThreadPoolExecutor
from threading import Lock

PHPMD_DEFAULT = 'text codesize,design,unusedcode'
cwd = os.path.dirname(__file__)


def main(argv):
    argv.pop(0)

    cmd = argv.pop(0) if len(argv) > 0 else ''
    cmds = cmd.split(',')

    for cmd in cmds:
        nline = 0
        if cmd == 'php-l':
            nline = exec(cwd+'/internal/php-lint-run')
        elif cmd == 'phan':
            nline = exec(cwd+'/internal/phan-run')
        elif cmd == 'phpmd':
            argv = PHPMD_DEFAULT if len(argv) == 0 else ' '.join(argv)
            nline = exec(cwd+'/internal/phpmd-run '+argv)
        elif cmd == 'multi':
            nline = linterMulti(argv)
        elif cmd == 'help':
            nline = usage()
        else:
            nline = 1
            print('unknown command:{}'.format(cmd))

        if nline > 0:
            break


def usage():
    str = '''
usage) {} [<mode> | help]

mode:
  php-l   linter by php-lint [default]
  phan    linter by phan
  phpmd   linter by phpmd
  multi   multiple linter(phan, phpmd, and php-l)

help: this message
    '''
    print(str.format(os.path.basename(__file__)))


def linterMulti(phpmdArgv):
    # php -l
    # NOTE: これは速いので先に済ませる
    nline = exec(cwd+'/internal/php-lint-run')
    if nline > 0:
        return nline

    # NOTE: 以降は時間がかかるので並列実行

    argv = PHPMD_DEFAULT if len(phpmdArgv) == 0 else ' '.join(phpmdArgv)
    # phpmd
    cmd_phpmd = cwd+'/internal/phpmd-run '+argv
    # phan
    cmd_phan = cwd+'/internal/phan-run'
    with ThreadPoolExecutor(max_workers=2) as executor:
        msgsList = executor.map(parallelCmd, [cmd_phpmd, cmd_phan])

    for msgs in msgsList:
        if len(msgs) > 0:
            for msg in msgs:
                print(msg)
            return len(msgs)
    return 0

def parallelCmd(cmd):
    lines = []

    proc = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE)
    while True:
        line = proc.stdout.readline()
        if not line:
            break
        lines.append(line.decode('utf-8').rstrip())

    return lines


# @return 行数
def exec(cmd):
    nline = 0
    proc = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE)
    buf = []

    while True:
        line = proc.stdout.readline()
        if not line:
            break

        print(line.decode('utf-8').rstrip())
        nline += 1

    return nline


if __name__ == '__main__':
    main(sys.argv[:])
