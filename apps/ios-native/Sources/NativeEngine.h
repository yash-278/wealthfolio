#ifndef WF_NATIVE_ENGINE_H
#define WF_NATIVE_ENGINE_H
#include <stdint.h>
char *wf_native_open(const char *directory);
char *wf_native_request(const char *command);
char *wf_native_stream(const char *command, int32_t (*callback)(const char *, void *), void *context);
void wf_native_free(char *reply);
#endif
