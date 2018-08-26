/*
 - (struct) webview
 - Arc<webview> -> set values, initialize
 - ...->set_userdata = { userdata, external_invoke, storage } (leaked Box?)
 - freed, when Arc<webview> gets dropped


*/