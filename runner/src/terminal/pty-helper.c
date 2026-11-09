// The Terminal panel's pseudo-terminal helper (docs/protocol.md, "The
// Terminal panel"). Node has no pseudo-terminal of its own, so the runner
// compiles this with the system's cc the first time a terminal opens.
//
//   pty-helper <cols> <rows> <cwd> <shell> [args...]
//
// It starts the shell on a new pseudo-terminal in <cwd>, copies what the
// shell writes to stdout, and reads frames from stdin: one type byte, a
// 4-byte big-endian length, then the payload. Type 0 is input for the
// shell; type 1 is a resize, two big-endian 16-bit numbers (cols, rows).
// When stdin closes (the runner stopped, or ended the terminal) it hangs
// the shell up. It exits with the shell's exit code, or 128 + the signal.
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <termios.h>
#include <unistd.h>
#if defined(__APPLE__)
#include <util.h>
#else
#include <pty.h>
#endif

#define MAX_FRAME (1 << 20)

static int write_all(int fd, const unsigned char *p, size_t n) {
  while (n > 0) {
    ssize_t w = write(fd, p, n);
    if (w < 0) {
      if (errno == EINTR) continue;
      if (errno == EAGAIN) {
        struct pollfd pf = {fd, POLLOUT, 0};
        poll(&pf, 1, 100);
        continue;
      }
      return -1;
    }
    p += w;
    n -= (size_t)w;
  }
  return 0;
}

static int clamp(long v) { return v < 1 ? 1 : v > 1000 ? 1000 : (int)v; }

static int exit_code(int status) {
  if (WIFEXITED(status)) return WEXITSTATUS(status);
  if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);
  return 1;
}

int main(int argc, char **argv) {
  if (argc < 5) {
    fprintf(stderr, "usage: pty-helper <cols> <rows> <cwd> <shell> [args...]\n");
    return 2;
  }
  struct winsize ws;
  memset(&ws, 0, sizeof ws);
  ws.ws_col = (unsigned short)clamp(strtol(argv[1], NULL, 10));
  ws.ws_row = (unsigned short)clamp(strtol(argv[2], NULL, 10));

  int master = -1;
  pid_t pid = forkpty(&master, NULL, NULL, &ws);
  if (pid < 0) {
    perror("forkpty");
    return 1;
  }
  if (pid == 0) {
    if (chdir(argv[3]) != 0) {
      fprintf(stderr, "pty-helper: can't open %s: %s\n", argv[3], strerror(errno));
    }
    setenv("TERM", "xterm-256color", 1);
    setenv("COLORTERM", "truecolor", 1);
    signal(SIGPIPE, SIG_DFL);
    execvp(argv[4], &argv[4]);
    fprintf(stderr, "pty-helper: can't start %s: %s\n", argv[4], strerror(errno));
    _exit(127);
  }

  signal(SIGPIPE, SIG_IGN);
  fcntl(STDOUT_FILENO, F_SETFL, fcntl(STDOUT_FILENO, F_GETFL) & ~O_NONBLOCK);

  unsigned char out[65536];
  unsigned char *frame = malloc(MAX_FRAME + 5);
  size_t have = 0;
  int stdin_open = 1;

  for (;;) {
    struct pollfd fds[2] = {{master, POLLIN, 0}, {STDIN_FILENO, POLLIN, 0}};
    int n = poll(fds, stdin_open ? 2 : 1, 1000);
    if (n < 0) {
      if (errno == EINTR) continue;
      break;
    }
    if (fds[0].revents & (POLLIN | POLLHUP | POLLERR)) {
      ssize_t r = read(master, out, sizeof out);
      if (r > 0) {
        if (write_all(STDOUT_FILENO, out, (size_t)r) != 0) break;
      } else if (r == 0 || (r < 0 && errno != EINTR && errno != EAGAIN)) {
        break;  // the shell and everything holding its terminal have gone
      }
    }
    if (stdin_open && (fds[1].revents & (POLLIN | POLLHUP | POLLERR))) {
      ssize_t r = read(STDIN_FILENO, frame + have, MAX_FRAME + 5 - have);
      if (r <= 0) {
        if (r < 0 && errno == EINTR) continue;
        // The runner has let go: hang the shell up, as closing a terminal
        // window does.
        stdin_open = 0;
        kill(-pid, SIGHUP);
        kill(pid, SIGHUP);
        break;
      }
      have += (size_t)r;
      for (;;) {
        if (have < 5) break;
        uint32_t len = ((uint32_t)frame[1] << 24) | ((uint32_t)frame[2] << 16) | ((uint32_t)frame[3] << 8) | frame[4];
        if (len > MAX_FRAME) {
          fprintf(stderr, "pty-helper: frame too big\n");
          kill(-pid, SIGHUP);
          goto done;
        }
        if (have < 5 + len) break;
        if (frame[0] == 0) {
          if (write_all(master, frame + 5, len) != 0) goto done;
        } else if (frame[0] == 1 && len == 4) {
          ws.ws_col = (unsigned short)clamp((frame[5] << 8) | frame[6]);
          ws.ws_row = (unsigned short)clamp((frame[7] << 8) | frame[8]);
          ioctl(master, TIOCSWINSZ, &ws);
        }
        memmove(frame, frame + 5 + len, have - 5 - len);
        have -= 5 + len;
      }
    }
  }

done:;
  int status = 0;
  // Give the shell a moment to go after a hang-up, then make sure.
  for (int i = 0; i < 20; i++) {
    pid_t w = waitpid(pid, &status, WNOHANG);
    if (w == pid) return exit_code(status);
    usleep(50000);
  }
  kill(-pid, SIGKILL);
  kill(pid, SIGKILL);
  if (waitpid(pid, &status, 0) == pid) return exit_code(status);
  return 1;
}
