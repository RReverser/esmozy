extern crate env_logger;

#[macro_use]
extern crate js;

use js::rust::{Runtime, Trace, SIMPLE_GLOBAL_CLASS};
use js::jsval::*;
use js::jsapi::*;
use js::conversions::*;
use std::string::String;

use std::ptr;

#[derive(Default)]
struct HeapValues {
    global: Heap<*mut JSObject>,
    parse: Heap<*mut JSFunction>,
}

impl Drop for HeapValues {
    fn drop(&mut self) {
        self.global.set(ptr::null_mut());
        self.parse.set(ptr::null_mut());
    }
}

unsafe impl Trace for HeapValues {
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        self.global.trace(tracer);
        self.parse.trace(tracer);
    }
}

extern fn trace_heap_values(tracer: *mut JSTracer, data: *mut ::std::os::raw::c_void) {
    unsafe {
        (*(data as *mut HeapValues)).trace(tracer);
    }
}

thread_local!(static RT: (Box<HeapValues>, Runtime) = unsafe {
    let rt = Runtime::new().unwrap();
    let cx = rt.cx();

    let mut heap_values = Box::<HeapValues>::default();

    JS_AddExtraGCRootsTracer(rt.rt(), Some(trace_heap_values), heap_values.as_mut() as *mut _ as _);

    heap_values.global.set(JS_NewGlobalObject(
        cx,
        &SIMPLE_GLOBAL_CLASS,
        ptr::null_mut(),
        OnNewGlobalHookOption::DontFireOnNewGlobalHook,
        &CompartmentOptions::default()
    ));

    let _ac = JSAutoCompartment::new(cx, heap_values.global.get());

    let global_handle = heap_values.global.handle();

    JS_InitReflectParse(cx, global_handle);
    JS_FireOnNewGlobalObject(cx, global_handle);

    rooted!(in(cx) let mut rval = UndefinedValue());

    rt.evaluate_script(
        global_handle,
        include_str!("script.js"),
        "script.js",
        1,
        rval.handle_mut()
    ).unwrap();

    heap_values.parse.set(JS_ValueToFunction(cx, rval.handle()));

    (heap_values, rt)
});

fn parse(input: &str) -> String {
    RT.with(|&(ref heap_values, ref rt)| {
        let cx = rt.cx();

        unsafe {
            let _ac = JSAutoCompartment::new(cx, heap_values.global.get());

            rooted!(in(cx) let mut source = UndefinedValue());
            input.to_jsval(cx, source.handle_mut());

            rooted!(in(cx) let mut rval = UndefinedValue());

            JS_CallFunction(
                cx,
                HandleObject::null(),
                heap_values.parse.handle(),
                &HandleValueArray::from_rooted_slice(&[source.get()]) as _,
                rval.handle_mut()
            );

            let mut out_json = String::new();

            unsafe extern fn json_writer(buf: *const u16, len: u32, data: *mut ::std::os::raw::c_void) -> bool {
                use std::{char, slice};

                let out_json = &mut *(data as *mut String);

                out_json.extend(
                    char::decode_utf16(
                        slice::from_raw_parts(buf, len as usize)
                        .iter()
                        .cloned()
                    )
                    .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                );

                true
            }

            rooted!(in(cx) let rval = rval.to_object());

            ToJSONMaybeSafely(
                cx,
                rval.handle(),
                Some(json_writer),
                &mut out_json as *mut _ as _
            );

            out_json
        }
    })
}

fn main() {
    env_logger::init().unwrap();

    if cfg!(debug_assertions) {
        RT.with(|&(.., ref rt)| unsafe {
            ::js::jsapi::JS_GC(rt.rt());
        });
    }

    println!("{}", parse("/abc/"));
}
