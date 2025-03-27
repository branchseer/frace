#include <mutex>
#include <fcntl.h>
#include <stdio.h>
#include <stdarg.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/un.h>

// https://github.com/apple-opensource/dyld/blob/e3f88907bebb8421f50f0943595f6874de70ebe0/include/mach-o/dyld-interposing.h
#define DYLD_INTERPOSE(_replacement,_replacee) \
   __attribute__((used)) static struct{ const void* replacement; const void* replacee; } _interpose_##_replacee \
            __attribute__ ((section ("__DATA,__interpose"))) = { (const void*)(unsigned long)&_replacement, (const void*)(unsigned long)&_replacee };


extern "C" {
   static int get_ipc_socket() {
      static std::once_flag flag;
      static int socket_fd;
      std::call_once(flag, []() {
         char* socket_path = getenv("FRACE_IPC");
         if (socket_path == NULL) {
            fprintf(stderr, "frace: env FRACE_IPC doesn't exist\n");
            abort();
         }

         socket_fd = socket(AF_UNIX, SOCK_DGRAM, 0);
         sockaddr_un addr;

         addr.sun_family = AF_UNIX;
         strcpy(addr.sun_path, socket_path);

         int ret = connect(socket_fd, (struct sockaddr *)&addr, sizeof(addr));
         if (ret == -1) {
            fprintf(stderr, "frace: failed to connect to unix socket. path: %s, errno: %d, error message: %s\n",
               socket_path, errno, strerror(errno));
            abort();
         }
      });
      return socket_fd;
   }

   static int open_hook(const char* path, int flags, mode_t mode)
   {
      int sock = get_ipc_socket();

      iovec iov[2];
      iov[0].iov_base = (void*)&flags;
      iov[0].iov_len = sizeof(flags);
      iov[1].iov_base = (void*)path;
      iov[1].iov_len = strlen(path);

      msghdr msg = { 0 };
      msg.msg_iov = iov;
      msg.msg_iovlen = 2;

      int ret = sendmsg(sock, &msg, 0);
      if (ret == -1) {
         fprintf(stderr, "frace: failed writing to sockfd %d: %d %s\n", sock, ret, strerror(errno));
         abort();
      }
      return open(path, flags, mode);
   }

   DYLD_INTERPOSE(open_hook, open)
}
