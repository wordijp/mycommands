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
	sigCh := make(chan os.Signal)
	// NOTE: SIGPIPEはエミュレート(Windowsでは基本受信出来ない)
	//signal.Notify(sigCh, os.Interrupt, os.Kill, syscall.SIGPIPE) // SIGPIPE not work(Windows)
	signal.Notify(sigCh, os.Interrupt, os.Kill)

	ioStopCh := make(chan struct{})
	ioWG := sync.WaitGroup{}

	// コマンド start ---
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
	err = <-waitCh
	// コマンド end ---

	// NOTE: 標準[エラー]出力は強制終了時も同期を取る
	close(ioStopCh)
	ioWG.Wait()

	if err != nil {
		if err2, ok := err.(*exec.ExitError); ok {
			if s, ok := err2.Sys().(syscall.WaitStatus); ok {
				err = nil
				exitCode = s.ExitStatus()
			}
		}
	}

	return
}

func doIO(out *os.File, in io.Reader, waitCh *chan error, cmd *exec.Cmd, ioStopCh chan struct{}, ioWG *sync.WaitGroup) {
	ioWG.Add(1)
	defer func() { ioWG.Done() }()

	scanner := bufio.NewScanner(in)
	for scanner.Scan() {
		select {
		case <-ioStopCh:
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
