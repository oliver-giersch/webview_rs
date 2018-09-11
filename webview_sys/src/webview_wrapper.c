#define WEBVIEW_IMPLEMENTATION

#include "webview.h"
#include <stdalign.h>
#include <stddef.h>

size_t struct_webview_size() { return sizeof(struct webview); }

size_t struct_webview_alignment() { return alignof(struct webview); }

size_t struct_webview_priv_size() { return sizeof(struct webview_priv); }

size_t struct_webview_priv_alignment() { return alignof(struct webview_priv); }

void struct_webview_set_title(struct webview *webview, const char *title) {
  webview->title = title;
}

void struct_webview_set_url(struct webview *webview, const char *url) {
  webview->url = url;
}

void struct_webview_set_width(struct webview *webview, int width) {
  webview->width = width;
}

void struct_webview_set_height(struct webview *webview, int height) {
  webview->height = height;
}

void struct_webview_set_resizable(struct webview *webview, int resizable) {
  webview->resizable = resizable;
}

void struct_webview_set_debug(struct webview *webview, int debug) {
  webview->debug = debug;
}

void struct_webview_set_external_invoke_cb(
    struct webview *webview, webview_external_invoke_cb_t external_invoke_cb) {
  webview->external_invoke_cb = external_invoke_cb;
}