#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winsock2.h>
#include <ws2tcpip.h> /* gai_strerror() */
#include <io.h> /* write() */
#include <fcntl.h> /* O_BINARY */
#else
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <fcntl.h>
#endif /* _WIN32 */

#include <sys/types.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <errno.h>
#include <errno.h>
#include <pthread.h>
#include <time.h>

#ifdef _WIN32
#define sleep(s) Sleep(1000 * (s))
#define read(s, buf, n) recv(s, buf, n, 0)
#define close(s) closesocket(s)
#define bzero(buf, n) memset(buf, '\0', n)

/* Hacks for 'errno' stuff
 */
#undef EAGAIN
#define EAGAIN WSAEWOULDBLOCK
#undef EWOULDBLOCK
#define EWOULDBLOCK WSAEWOULDBLOCK
#undef errno
#define errno WSAGetLastError()
#define perror(str) fprintf(stderr, str ": %d.\n", WSAGetLastError())
#define strerror(e) ws_strerror(e)
#ifndef STDOUT_FILENO
#define STDOUT_FILENO 1 /* MinGW has this */
#endif /* !STDOUT_FILENO */
#endif /* _WIN32 */

/* crustls.h is autogenerated in the Makefile using cbindgen. */
#include "crustls.h"

enum crustls_demo_result
{
  CRUSTLS_DEMO_OK,
  CRUSTLS_DEMO_ERROR,
  CRUSTLS_DEMO_AGAIN,
  CRUSTLS_DEMO_EOF,
  CRUSTLS_DEMO_CLOSE_NOTIFY,
};

typedef struct conndata_t {
  int fd;
  struct rustls_connection *rconn;
  char *data_from_client;
  size_t data_len;
  size_t data_capacity;
} conndata_t;

void
print_error(char *prefix, rustls_result result)
{
  char buf[256];
  size_t n;
  rustls_error(result, buf, sizeof(buf), &n);
  fprintf(stderr, "%s: %.*s\n", prefix, (int)n, buf);
}

#ifdef _WIN32
const char *
ws_strerror(int err)
{
  static char ws_err[50];

  if(err >= WSABASEERR) {
    snprintf(ws_err, sizeof(ws_err), "Winsock err: %d", err);
    return ws_err;
  }
  /* Assume a CRT error */
  return (strerror)(err);
}
#endif

/*
 * Set a socket to be nonblocking.
 *
 * Returns CRUSTLS_DEMO_OK on success, CRUSTLS_DEMO_ERROR on error.
 */
enum crustls_demo_result
nonblock(int sockfd)
{
#ifdef _WIN32
  u_long nonblock = 1UL;

  if(ioctlsocket(sockfd, FIONBIO, &nonblock) != 0) {
    perror("Error setting socket nonblocking");
    return CRUSTLS_DEMO_ERROR;
  }
#else
  int flags;
  flags = fcntl(sockfd, F_GETFL, 0);
  if(flags < 0) {
    perror("getting socket flags");
    return CRUSTLS_DEMO_ERROR;
  }
  flags = fcntl(sockfd, F_SETFL, flags | O_NONBLOCK);
  if(flags < 0) {
    perror("setting socket nonblocking");
    return CRUSTLS_DEMO_ERROR;
  }
#endif
  return CRUSTLS_DEMO_OK;
}

enum crustls_demo_result
read_file(const char *filename, char *buf, size_t buflen, size_t *n)
{
  FILE *f = fopen(filename, "r");
  if(f == NULL) {
    fprintf(stderr, "%s\n", strerror(errno));
    return CRUSTLS_DEMO_ERROR;
  }
  *n = fread(buf, 1, buflen, f);
  if(!feof(f)) {
    fprintf(stderr, "%s\n", strerror(errno));
    return CRUSTLS_DEMO_ERROR;
  }
  return CRUSTLS_DEMO_OK;
}

int read_cb(void *userdata, uint8_t *buf, uintptr_t len, uintptr_t *out_n)
{
  ssize_t n = 0;
  struct conndata_t *conn = (struct conndata_t*)userdata;
  n = read(conn->fd, buf, len);
  if(n < 0) {
    return errno;
  }
  if (out_n != NULL) {
    *out_n = n;
  }
  return 0;
}

int write_cb(void *userdata, const uint8_t *buf, uintptr_t len, uintptr_t *out_n)
{
  ssize_t n = 0;
  struct conndata_t *conn = (struct conndata_t*)userdata;

#ifdef _WIN32
  n = send(conn->fd, buf, len);
#else
  n = write(conn->fd, buf, len);
#endif
  if(n < 0) {
    return errno;
  }
  *out_n = n;
  return 0;
}

/*
 * Write n bytes from buf to the provided fd, retrying short writes until
 * we finish or hit an error. Assumes fd is blocking and therefore doesn't
 * handle EAGAIN. Returns 0 for success or 1 for error.
 *
 * For Winsock we cannot use a socket-fd in write().
 * Call send() if fd > STDOUT_FILENO.
 */
int
write_all(int fd, const char *buf, int n)
{
  int m = 0;

  while(n > 0) {
    m = write(fd, buf, n);
    if(m < 0) {
      perror("writing to stdout");
      return 1;
    }
    if(m == 0) {
      fprintf(stderr, "early EOF when writing to stdout\n");
      return 1;
    }
    n -= m;
  }
  return 0;
}

/* Read all available bytes from the rustls_connection until EOF.
 * Note that EOF here indicates "no more bytes until
 * process_new_packets", not "stream is closed".
 *
 * Returns CRUSTLS_DEMO_OK for success,
 * CRUSTLS_DEMO_ERROR for error,
 * CRUSTLS_DEMO_CLOSE_NOTIFY for "received close_notify"
 */
int
copy_plaintext_to_buffer(struct conndata_t *conn)
{
  int result;
  size_t n;
  struct rustls_connection *rconn = conn->rconn;

  if (conn->data_capacity - conn->data_len < 1024) {
    conn->data_from_client = realloc(conn->data_from_client,
      conn->data_capacity * 2);
    if (conn->data_from_client == NULL) {
      fprintf(stderr, "out of memory\n");
      abort();
    }
  }

  for(;;) {
    char *buf = conn->data_from_client + conn->data_len;
    size_t avail = conn->data_capacity - conn->data_len - 1;
    result = rustls_connection_read(rconn, (uint8_t *)buf, avail, &n);
    if(result == RUSTLS_RESULT_ALERT_CLOSE_NOTIFY) {
      fprintf(stderr, "Received close_notify, cleanly ending connection\n");
      return CRUSTLS_DEMO_CLOSE_NOTIFY;
    }
    if(result != RUSTLS_RESULT_OK) {
      fprintf(stderr, "Error in ClientSession::read\n");
      return CRUSTLS_DEMO_ERROR;
    }
    if(n == 0) {
      /* This is expected. It just means "no more bytes for now." */
      return CRUSTLS_DEMO_OK;
    }
    conn->data_len += n;

    result = write_all(STDOUT_FILENO, buf, n);
    if(result != 0) {
      return CRUSTLS_DEMO_ERROR;
    }
  }

  return CRUSTLS_DEMO_ERROR;
}

typedef enum exchange_state {
  READING_REQUEST,
  SENT_RESPONSE
} exchange_state;

/*
 * Do one read from the socket, and process all resulting bytes into the
 * rustls_connection, then copy all plaintext bytes from the session to stdout.
 * Returns:
 *  - CRUSTLS_DEMO_OK for success
 *  - CRUSTLS_DEMO_AGAIN if we got an EAGAIN or EWOULDBLOCK reading from the
 *    socket
 *  - CRUSTLS_DEMO_EOF if we got EOF
 *  - CRUSTLS_DEMO_ERROR for other errors.
 */
enum crustls_demo_result
do_read(struct conndata_t *conn, struct rustls_connection *rconn)
{
  int err = 1;
  int result = 1;
  size_t n = 0;

  err = rustls_connection_read_tls(rconn, read_cb, conn, &n);
  if(err == EAGAIN || err == EWOULDBLOCK) {
    fprintf(stderr,
            "reading from socket: EAGAIN or EWOULDBLOCK: %s\n",
            strerror(errno));
    return CRUSTLS_DEMO_AGAIN;
  }
  else if(err != 0) {
    fprintf(stderr, "reading from socket: errno %d\n", err);
    return CRUSTLS_DEMO_ERROR;
  }

  if (n == 0) {
    return CRUSTLS_DEMO_EOF;
  }
  fprintf(stderr, "read %ld bytes from socket\n", n);

  result = rustls_connection_process_new_packets(rconn);
  if(result != RUSTLS_RESULT_OK) {
    print_error("in process_new_packets", result);
    return CRUSTLS_DEMO_ERROR;
  }

  result = copy_plaintext_to_buffer(conn);
  if(result != CRUSTLS_DEMO_CLOSE_NOTIFY) {
    fprintf(stderr, "do_read returning %d\n", result);
    return result;
  }

  char buf[2048];
  /* If we got a close_notify, verify that the sender then
   * closed the TCP connection. */
  n = read(conn->fd, buf, sizeof(buf));
  if(n != 0 && errno != EWOULDBLOCK) {
    fprintf(stderr, "read returned %ld after receiving close_notify: %s\n", n, strerror(errno));
    return CRUSTLS_DEMO_ERROR;
  }
  return CRUSTLS_DEMO_CLOSE_NOTIFY;
}

bool
request_is_finished(struct conndata_t *conn) {
   conn->data_from_client[conn->data_len] = 0;
   return strstr(conn->data_from_client, "\r\n\r\n") != NULL;
}

void
send_response(struct conndata_t *conn) {
  struct rustls_connection *rconn = conn->rconn;
  const char* response = "HTTP/1.1 200 OK\r\nContent-Length: 6\r\n\r\nhello\n";
  size_t n;
  rustls_connection_write(rconn, response, strlen(response), &n);
  if (n != strlen(response)) {
    fprintf(stderr, "failed to write all response bytes. wrote %ld\n", n);
    abort();
  }
}

void *
handle_conn(void *userdata) {
  int ret = 1;
  int err = 1;
  int result = 1;
  char buf[2048];
  fd_set read_fds;
  fd_set write_fds;
  size_t n = 0;
  conndata_t *conn = userdata;
  struct rustls_connection *rconn = conn->rconn;
  int sockfd = conn->fd;
  struct timespec ts;
  enum exchange_state state = READING_REQUEST;

  fprintf(stderr, "accepted conn on fd %d\n", conn->fd);

  for(;;) {
    FD_ZERO(&read_fds);
    if (rustls_connection_wants_read(rconn)) {
      FD_SET(sockfd, &read_fds);
    }
    FD_ZERO(&write_fds);
    if (rustls_connection_wants_write(rconn)) {
      FD_SET(sockfd, &write_fds);
    }

    result = select(sockfd + 1, &read_fds, &write_fds, NULL, NULL);
    if(result == -1) {
      perror("select");
      goto cleanup;
    }
    if(result == 0) {
      fprintf(stderr, "no fds from select, sleeping\n");
      ts.tv_sec = 0;
      ts.tv_nsec = 1000000000;
      nanosleep(&ts, NULL);
    }

    if(FD_ISSET(sockfd, &read_fds)) {
      fprintf(stderr,
              "rustls wants us to read_tls. First we need to pull some "
              "bytes from the socket\n");

      /* Read all bytes until we get EAGAIN. Then loop again to wind up in
         select awaiting the next bit of data. */
      for(;;) {
        result = do_read(conn, rconn);
        if(result == CRUSTLS_DEMO_AGAIN) {
          break;
        }
        else if(result == CRUSTLS_DEMO_CLOSE_NOTIFY) {
          ret = 0;
          goto cleanup;
        }
        else if(result != CRUSTLS_DEMO_OK) {
          goto cleanup;
        }
      }
    }
    if(FD_ISSET(sockfd, &write_fds)) {
      fprintf(stderr, "rustls wants us to write_tls.\n");
      err = rustls_connection_write_tls(rconn, write_cb, conn, &n);
      if(err != 0) {
        fprintf(stderr, "Error in write_tls: errno %d\n", err);
        goto cleanup;
      }
      else if(n == 0) {
        fprintf(stderr, "EOF from write_tls\n");
        goto cleanup;
      }
    }

    if (state == READING_REQUEST && request_is_finished(conn)) {
      state = SENT_RESPONSE;
      fprintf(stderr, "writing response\n");
      send_response(conn);
    }
  }

  fprintf(stderr, "handle_conn: loop fell through");

cleanup:
  if(sockfd > 0) {
    close(sockfd);
  }
}

int
main(int argc, const char **argv)
{
  int ret = 1;
  int result = 1;
  const char *certfile = argv[1];
  const char *keyfile = argv[2];
  char certbuf[10000];
  size_t certbuf_len;
  char keybuf[10000];
  size_t keybuf_len;
  struct rustls_server_config_builder *config_builder =
    rustls_server_config_builder_new();
  const struct rustls_server_config *server_config = NULL;
  struct rustls_connection *rconn = NULL;

  env_logger_init();

  if(argc <= 2) {
    fprintf(stderr,
            "usage: %s cert.pem key.pem\n\n"
            "Listen on port 8443 with the given cert and key.\n",
            argv[0]);
    goto cleanup;
  }

  result = read_file(certfile, certbuf, sizeof(certbuf), &certbuf_len);
  if(result != CRUSTLS_DEMO_OK) {
    goto cleanup;
  }

  result = read_file(keyfile, keybuf, sizeof(keybuf), &keybuf_len);
  if(result != CRUSTLS_DEMO_OK) {
    goto cleanup;
  }

  const struct rustls_certified_key *certified_key;
  result = rustls_certified_key_build(
    certbuf, certbuf_len, keybuf, keybuf_len, &certified_key);
  if(result != RUSTLS_RESULT_OK) {
    print_error("parsing certificate and key", result);
    return 1;
  }

  rustls_server_config_builder_set_certified_keys(config_builder, &certified_key, 1);
  server_config = rustls_server_config_builder_build(config_builder);

#ifdef _WIN32
  WSADATA wsa;
  WSAStartup(MAKEWORD(1, 1), &wsa);
#endif

  int sockfd = socket(AF_INET, SOCK_STREAM, 0);
  if(sockfd < 0) {
    fprintf(stderr, "making socket: %s", strerror(errno));
  }

  struct sockaddr_in my_addr, peer_addr;
  memset(&my_addr, 0, sizeof(struct sockaddr_in));
  /* Clear structure */
  my_addr.sin_family = AF_INET;
  my_addr.sin_addr.s_addr = INADDR_ANY;
  my_addr.sin_port = htons(8443);

  if(bind(sockfd, (struct sockaddr *)&my_addr, sizeof(struct sockaddr_in)) ==
     -1) {
    perror("bind");
    goto cleanup;
  }

  if(listen(sockfd, 50) == -1) {
    perror("listen");
    goto cleanup;
  }
  fprintf(stderr, "listening on localhost:8443\n");

  while (true) {
    socklen_t peer_addr_size;
    peer_addr_size = sizeof(struct sockaddr_in);
    int clientfd =
      accept(sockfd, (struct sockaddr *)&peer_addr, &peer_addr_size);
    if(clientfd < 0) {
      perror("accept");
      goto cleanup;
    }

    nonblock(clientfd);

    result = rustls_server_connection_new(server_config, &rconn);
    if(result != RUSTLS_RESULT_OK) {
      print_error("making session", result);
      goto cleanup;
    }

    pthread_t thrd;
    conndata_t *conndata;
    conndata = calloc(1, sizeof(conndata_t));
    conndata->fd = clientfd;
    conndata->rconn = rconn;
    conndata->data_from_client = calloc(1, 2048);
    conndata->data_capacity = 2048;
    ret = pthread_create(&thrd, NULL, handle_conn, conndata);
    if (ret != 0) {
      fprintf(stderr, "error from pthread_create: %d\n", ret);
      goto cleanup;
    }
    pthread_join(thrd, NULL);
    if (ret != 0) {
      fprintf(stderr, "error from pthread_join: %d\n", ret);
    }
  }

  // Success!
  ret = 0;

cleanup:
  rustls_server_config_free(server_config);
  rustls_connection_free(rconn);

#ifdef _WIN32
  WSACleanup();
#endif

  return ret;
}
