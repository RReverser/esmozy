extern crate env_logger;

#[macro_use]
extern crate js;

use js::conversions::{FromJSValConvertible, ToJSValConvertible, ConversionResult};
use js::jsapi::CompartmentOptions;
use js::jsapi::JS_NewGlobalObject;
use js::jsapi::OnNewGlobalHookOption;
use js::jsapi::JS_InitReflectParse;
use js::jsapi::JSAutoCompartment;
use js::jsapi::JS_FireOnNewGlobalObject;
use js::jsval::{UndefinedValue, ObjectValue};
use js::jsapi::HandleObject;
use js::rust::{Runtime, SIMPLE_GLOBAL_CLASS};
use js::jsapi::ToJSONMaybeSafely;

use std::ptr;

thread_local!(static RT: Runtime = Runtime::new().unwrap());

fn parse(input: &str) -> String {
    RT.with(|rt| {
        let cx = rt.cx();

        unsafe {
            rooted!(in(cx) let global = JS_NewGlobalObject(
                cx,
                &SIMPLE_GLOBAL_CLASS,
                ptr::null_mut(),
                OnNewGlobalHookOption::DontFireOnNewGlobalHook,
                &CompartmentOptions::default()
            ));
            let global = global.handle();
            let _ac = JSAutoCompartment::new(cx, global.get());
            JS_InitReflectParse(cx, global);
            JS_FireOnNewGlobalObject(cx, global);
            rooted!(in(cx) let mut source = UndefinedValue());
            input.to_jsval(cx, source.handle_mut());
            ::js::jsapi::JS_SetProperty(cx, global, b"source\0" as *const _ as _, source.handle());
            rooted!(in(cx) let mut rval = UndefinedValue());
            assert!(rt.evaluate_script(global, r#"
                try {
                    Reflect.parse(source)
                } catch (e) {
                    ({type: "Error", message: e.message})
                }
            "#, "test", 1, rval.handle_mut()).is_ok());
            rooted!(in(cx) let rval = rval.to_object());

            let mut out_json = String::new();

            unsafe extern fn json_writer(buf: *const u16, len: u32, data: *mut ::std::os::raw::c_void) -> bool {
                use std::{char, slice};

                let out_json = &mut *(data as *mut String);

                out_json.extend(
                    char::decode_utf16(
                        ::std::slice::from_raw_parts(buf, len as usize)
                        .iter()
                        .cloned()
                    )
                    .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                );

                true
            }

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

    println!("{}", parse("hello + '\\uD834\\uDF06'"));
}
