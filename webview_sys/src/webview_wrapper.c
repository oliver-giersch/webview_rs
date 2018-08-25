#define WEBVIEW_IMPLEMENTATION

#include <stddef.h>
#include "webview.h"

/*struct webview* alloc_webview(void) {
    struct webview* webview = calloc(1, sizeof(struct webview));
    return webview;
}

void free_webview(struct webview* webview) {
    free(webview);
}*/

size_t struct_webview_size() {
    return sizeof(struct webview);
}

size_t struct_webview_priv_size() {
    return sizeof(struct webview_priv);
}

void struct_webview_set_title(struct webview* webview, const char* title) {
    webview->title = title;
}

void struct_webview_set_url(struct webview* webview, const char* url) {
    webview->url = url;
}

void struct_webview_set_width(struct webview* webview, int width) {
    webview->width = width;
}

void struct_webview_set_height(struct webview* webview, int height) {
    webview->height = height;
}

void struct_webview_set_resizable(struct webview* webview, int resizable) {
    webview->resizable = resizable;
}

void struct_webview_set_debug(struct webview* webview, int debug) {
    webview->debug = debug;
}

void struct_webview_set_external_invoke_cb(struct webview* webview, webview_external_invoke_cb_t external_invoke_cb) {
    webview->external_invoke_cb = external_invoke_cb;
}

/*void struct_webview_set_userdata(struct webview* webview, void* userdata) {
    webview->userdata = userdata;
}*/