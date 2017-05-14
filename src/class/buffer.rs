// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::os::raw::c_int;

use ffi;
use err::{PyErr, PyResult};
use python::{self, Python, PythonObject};
use conversion::ToPyObject;
use objects::{PyObject, PyType, PyModule};
use py_class::slots::UnitCallbackConverter;
use function::handle_callback;


pub trait PyBufferProtocolImpl {
    fn methods() -> &'static [&'static str];
}

impl<T> PyBufferProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        static METHODS: &'static [&'static str] = &[];
        METHODS
    }
}

pub trait PyBufferProtocol {

    fn bf_getbuffer(&self, py: Python, view: *mut ffi::Py_buffer, flags: c_int) -> PyResult<()>;

    fn bf_releasebuffer(&self, py: Python, view: *mut ffi::Py_buffer) -> PyResult<()>;

}

impl<T> PyBufferProtocol for T {

    default fn bf_getbuffer(&self, _py: Python,
                            _view: *mut ffi::Py_buffer, _flags: c_int) -> PyResult<()> {
        Ok(())
    }
    default fn bf_releasebuffer(&self, _py: Python,
                                _view: *mut ffi::Py_buffer) -> PyResult<()> {
        Ok(())
    }
}


impl ffi::PyBufferProcs {

    pub fn new<T>() -> Option<ffi::PyBufferProcs>
        where T: PyBufferProtocol + PyBufferProtocolImpl + PythonObject
    {
        let methods = T::methods();
        if methods.is_empty() {
            return None
        }

        let mut buf_procs: ffi::PyBufferProcs = ffi::PyBufferProcs_INIT;

        for name in methods {
            match name {
                &"bf_getbuffer" => {
                    buf_procs.bf_getbuffer = {
                        unsafe extern "C" fn wrap<T>(slf: *mut ffi::PyObject, arg1: *mut ffi::Py_buffer, arg2: c_int) -> c_int
                            where T: PyBufferProtocol + PythonObject
                        {
                            const LOCATION: &'static str = concat!(stringify!(T), ".buffer_get::<PyBufferProtocol>()");
                            handle_callback(LOCATION, UnitCallbackConverter,
                                            |py| {
                                                let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
                                                let result = slf.bf_getbuffer(py, arg1, arg2);
                                                ::PyDrop::release_ref(slf, py);
                                                result
                                            }
                            )
                        }
                        Some(wrap::<T>)
                    }
                },
                _ => ()
            }
        }

        Some(buf_procs)
    }
}