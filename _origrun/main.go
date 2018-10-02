// バッチファイル終了時の「バッチ ジョブを終了しますか」を出さないのが目的

package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"os/exec"
	"os/signal"
	"regexp"
	"sync"
	"syscall"
)

var pattern = regexp.MustCompile(`.exe$`)

func main() {
	// コマンドの実行
	bin := pattern.ReplaceAllString(os.Args[0], "")
	args := append([]string{"/c", "call", bin + ".bat"}, os.Args[1:]...)
	cmd := exec.Command("cmd", args...)

	exitCode, err := runCommand(cmd)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
	}

	os.Exit(exitCode)
}

func runCommand(cmd *exec.Cmd) (exitCode int, err error) {
	// 外部コマンドとの標準入[エラー]出力を繋げる
	oRead, err := cmd.StdoutPipe()
	if err != nil {
		return
	}
	eRead, err := cmd.StderrPipe()
	if err != nil {
		return
	}
	cmd.Stdin = os.Stdin

	waitCh := make(chan error)
	// シグナル受信
	sigCh := make(chan os.Signal)
	// NOTE: SIGPIPEはエミュレート(Windowsでは基本受信出来ない)
	//signal.Notify(sigCh, os.Interrupt, os.Kill, syscall.SIGPIPE) // SIGPIPE not work(Windows)
	signal.Notify(sigCh, os.Interrupt, os.Kill)

	// 終了待機(標準[エラー]出力用)
	ioStopCh := make(chan struct{})
	ioWG := sync.WaitGroup{}

	// コマンド start
	if err = cmd.Start(); err != nil {
		return
	}
	go func() {
		waitCh <- cmd.Wait()
	}()
	go func() {
		for {
			switch <-sigCh {
			case os.Interrupt:
				exitCode = int(syscall.SIGINT)
				waitCh <- cmd.Process.Kill()
				return
			case os.Kill:
				exitCode = int(syscall.SIGKILL)
				waitCh <- cmd.Process.Kill()
				return
			default:
				// no-op
			}
		}
	}()
	go doIO(os.Stdout, oRead, &waitCh, cmd, ioStopCh, &ioWG)
	go doIO(os.Stderr, eRead, &waitCh, cmd, ioStopCh, &ioWG)
	// コマンド end
	err = <-waitCh

	// NOTE: 標準[エラー]出力は強制終了時も同期を取る
	close(ioStopCh)
	ioWG.Wait()

	if err != nil {
		// エラー検知
		if err2, ok := err.(*exec.ExitError); ok {
			// その中の、シグナル(SIGINT:2 等の)エラー検知
			if s, ok := err2.Sys().(syscall.WaitStatus); ok {
				err = nil
				exitCode = s.ExitStatus()
			}
		}
	}

	return
}

func doIO(out *os.File, in io.Reader, waitCh *chan error, cmd *exec.Cmd, stopCh chan struct{}, wg *sync.WaitGroup) {
	wg.Add(1)
	defer func() { wg.Done() }()

	scanner := bufio.NewScanner(in)
	for scanner.Scan() {
		// 終了済みか？
		select {
		case <-stopCh:
			return
		default:
			// no-op
		}

		_, err := fmt.Fprintln(out, scanner.Text())

		// NOTE: SIGPIPEエミュレート
		if err != nil {
			*waitCh <- cmd.Process.Kill()
			break
		}
	}
}
