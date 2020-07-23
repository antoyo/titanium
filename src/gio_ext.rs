use std::ptr;

use gio::{
    Cancellable,
    SocketConnection,
    SocketListener,
};
use gio_sys;
use glib::Error;
use glib::translate::{
    ToGlibPtr,
    from_glib_full,
    from_glib_none,
};
use glib_sys;
use gobject_sys;

// TODO: remove when https://github.com/gtk-rs/gio/issues/99 is fixed.
pub struct ListenerAsync {
    listener: SocketListener,
}

impl<'b> ListenerAsync {
    pub fn new(listener: SocketListener) -> Self {
        Self {
            listener,
        }
    }

    pub fn accept_async<'a, P: Into<Option<&'a Cancellable>>, Q: FnOnce(Result<(SocketConnection, Option<glib::Object>), Error>) + Send + 'static>(&self, cancellable: P, callback: Q) {
        let cancellable = cancellable.into();
        let cancellable = cancellable.to_glib_none();
        let user_data: Box<Box<Q>> = Box::new(Box::new(callback));
        unsafe extern "C" fn accept_async_trampoline<Q: FnOnce(Result<(SocketConnection, Option<glib::Object>), Error>) + Send + 'static>(_source_object: *mut gobject_sys::GObject, res: *mut gio_sys::GAsyncResult, user_data: glib_sys::gpointer)
        {
            let mut error = ptr::null_mut();
            let mut source_object = ptr::null_mut();
            let ret = gio_sys::g_socket_listener_accept_finish(_source_object as *mut _, res, &mut source_object, &mut error);
            let result = if error.is_null() {
                Ok((from_glib_full(ret),
                from_glib_none(source_object)))
            } else { Err(from_glib_full(error)) };
            let callback: Box<Box<Q>> = Box::from_raw(user_data as *mut _);
            callback(result);
        }
        let callback = accept_async_trampoline::<Q>;
        unsafe {
            gio_sys::g_socket_listener_accept_async(self.listener.to_glib_none().0, cancellable.0, Some(callback), Box::into_raw(user_data) as *mut _);
        }
    }
}
