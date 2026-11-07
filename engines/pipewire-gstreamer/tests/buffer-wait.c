/* Exercise the pinned pool's actual acquire predicate without a notification.
 * The stream's dequeue boundary is the only stand-in; no server is contacted. */
#include <stdatomic.h>
#include <stdlib.h>
#include <stdio.h>
#include "gstpipewirepool.c"

static atomic_int available;
static atomic_int attempted;
static atomic_int finished;
static struct pw_buffer offered;

struct pw_buffer *pw_stream_dequeue_buffer(struct pw_stream *stream)
{
  (void) stream;
  atomic_store(&attempted, 1);
  return atomic_exchange(&available, 0) ? &offered : NULL;
}

static gpointer make_available(gpointer unused)
{
  (void) unused;
  while (!atomic_load(&attempted))
    g_usleep(1000);
  g_usleep(10000);
  /* Deliberately do not signal: reproduce availability after a missed wake. */
  atomic_store(&available, 1);
  return NULL;
}

static gpointer deadline(gpointer unused)
{
  (void) unused;
  g_usleep(2000000);
  if (!atomic_load(&finished)) {
    fputs("BUFFER_WAIT_DID_NOT_RECHECK\n", stderr);
    _Exit(73);
  }
  return NULL;
}

int main(int argc, char **argv)
{
  gst_init(&argc, &argv);
  pw_init(&argc, &argv);
  GstPipeWireStream *stream = gst_pipewire_stream_new(NULL);
  GstPipeWirePool *pool = stream->pool;
  GstPipeWirePoolData data = { 0 };
  data.buf = gst_buffer_new();
  offered.user_data = &data;
  /* This isolates acquire_buffer from the public pool's allocation protocol. */
  g_atomic_int_set(&GST_BUFFER_POOL(pool)->flushing, FALSE);
  GThread *arrival = g_thread_new("available-without-signal", make_available, NULL);
  g_thread_unref(g_thread_new("wait-deadline", deadline, NULL));
  GstBuffer *received = NULL;
  GstBufferPoolAcquireParams params = { 0 };
  gint64 began = g_get_monotonic_time();
  GstFlowReturn result = acquire_buffer(GST_BUFFER_POOL(pool), &received, &params);
  atomic_store(&finished, 1);
  g_thread_join(arrival);
  if (result != GST_FLOW_OK || received != data.buf || g_get_monotonic_time() - began > 1500000)
    return 1;
  params.flags = GST_BUFFER_POOL_ACQUIRE_FLAG_DONTWAIT;
  if (acquire_buffer(GST_BUFFER_POOL(pool), &received, &params) != GST_FLOW_EOS)
    return 2;
  params.flags = GST_BUFFER_POOL_ACQUIRE_FLAG_LAST;
  pool->paused = TRUE;
  if (acquire_buffer(GST_BUFFER_POOL(pool), &received, &params) != GST_FLOW_CUSTOM_ERROR_1)
    return 3;
  g_atomic_int_set(&GST_BUFFER_POOL(pool)->flushing, TRUE);
  if (acquire_buffer(GST_BUFFER_POOL(pool), &received, &params) != GST_FLOW_FLUSHING)
    return 4;
  gst_buffer_unref(data.buf);
  gst_object_unref(stream);
  puts("BUFFER_WAIT_RECHECK_AND_CANCELLATION_PASSED");
  return 0;
}
