#![allow(missing_docs)]

use std::mem;
use std::ptr;

use gio::{
    Cancellable,
    OutputStream,
    SocketAddress,
    SocketConnection,
    SocketListener,
};
use gio_sys;
use glib::{self, Error, Priority};
use glib::translate::{
    FromGlibPtrFull,
    ToGlib,
    ToGlibPtr,
    from_glib_full,
    from_glib_none,
    mut_override,
};
use glib_sys;
use gobject_sys;

macro_rules! callback_guard {
    () => (
        let _guard = glib::CallbackGuard::new();
    )
}

pub fn new_abstract_socket_address(name: &[u8]) -> SocketAddress {
    let path = name.to_glib_none().0 as *mut i8;
    let len = name.len();
    unsafe {
        SocketAddress::from_glib_full(gio_sys::g_unix_socket_address_new_with_type(path, len as i32,
            gio_sys::G_UNIX_SOCKET_ADDRESS_ABSTRACT))
    }
}

pub trait WriteAsync {
    fn write_all_async<'a, B: AsRef<[u8]> + Send + 'static, P: Into<Option<&'a Cancellable>>, Q: FnOnce(Result<(B, usize, Option<Error>), (B, Error)>) + Send + 'static>(&self, buffer: B, io_priority: Priority, cancellable: P, callback: Q);
}

impl WriteAsync for OutputStream {
    fn write_all_async<'a, B: AsRef<[u8]> + Send + 'static, P: Into<Option<&'a Cancellable>>, Q: FnOnce(Result<(B, usize, Option<Error>), (B, Error)>) + Send + 'static>(&self, buffer: B, io_priority: Priority, cancellable: P, callback: Q) {
        let cancellable = cancellable.into();
        let cancellable = cancellable.to_glib_none();
        let buffer: Box<B> = Box::new(buffer);
        let (count, buffer_ptr) = {
            let slice = (*buffer).as_ref();
            (slice.len(), slice.as_ptr())
        };
        let user_data: Box<Option<(Box<Q>, Box<B>)>> = Box::new(Some((Box::new(callback), buffer)));
        unsafe extern "C" fn write_all_async_trampoline<B: AsRef<[u8]> + Send + 'static, Q: FnOnce(Result<(B, usize, Option<Error>), (B, Error)>) + Send + 'static>(_source_object: *mut gobject_sys::GObject, res: *mut gio_sys::GAsyncResult, user_data: glib_sys::gpointer)
        {
            callback_guard!();
            let mut user_data: Box<Option<(Box<Q>, Box<B>)>> = Box::from_raw(user_data as *mut _);
            let (callback, buffer) = user_data.take().unwrap();
            let buffer = *buffer;

            let mut error = ptr::null_mut();
            let mut bytes_written = mem::uninitialized();
            let _ = gio_sys::g_output_stream_write_all_finish(_source_object as *mut _, res, &mut bytes_written, &mut error);
            let result = if error.is_null() {
                Ok((buffer, bytes_written, None))
            } else if bytes_written != 0 {
                Ok((buffer, bytes_written, from_glib_full(error)))
            } else {
                Err((buffer, from_glib_full(error)))
            };
            callback(result);
        }
        let callback = write_all_async_trampoline::<B, Q>;
        unsafe {
            gio_sys::g_output_stream_write_all_async(self.to_glib_none().0, mut_override(buffer_ptr), count, io_priority.to_glib(), cancellable.0, Some(callback), Box::into_raw(user_data) as *mut _);
        }
    }
}

pub struct ListenerAsync<'a> {
    listener: &'a SocketListener,
}

impl<'b> ListenerAsync<'b> {
    pub fn new(listener: &'b SocketListener) -> Self {
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
            callback_guard!();
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
