
#![allow(non_snake_case)]

use std::os::raw::c_void;
use std::ffi::CStr;
use std::fs::File;
use std::io::prelude::*;

use jni::{JNIEnv, JavaVM};
use jni::objects::{JString, JObject, JFieldID, JValue};
use jni::signature::{JavaType, Primitive};
use jni::sys::*;

use adblock::engine::Engine;

const REQUEST_TYPES: [&str;11] = ["unknown", "script", "stylesheet", "object", "image", "font", "document", "media", "sub_frame", " websocket", "xmlhttprequest"];
static mut CACHED_ABC_NATIVE_THIS_FIELD_ID: Option<JFieldID> = None;


/// Produces `JFieldID` for a particular field dealing with its lifetime.
///
/// Always returns `Some(field_id)`, panics if field not found.
fn get_field_id(env: &JNIEnv, class: &str, name: &str, sig: &str) -> JFieldID<'static> {
    let field_id = env
        .get_field_id(class, name, sig)
        // we need this line to erase lifetime in order to save underlying raw pointer in static
        .map(|mid| mid.into_inner().into())
        .unwrap_or_else(|_| {
            panic!(
                "Field {} with signature {} of class {} not found",
                name, sig, class
            )
        });
    field_id
}

fn release_native_this_if_any(env: &JNIEnv, obj: &JObject) {
    unsafe {
        let field: jlong = env.get_field_unchecked(*obj, CACHED_ABC_NATIVE_THIS_FIELD_ID.unwrap(), JavaType::Primitive(Primitive::Long)).unwrap().j().unwrap();
        if field != 0 {
            let eng = field as *mut Engine;
            Box::from_raw(eng);//delete engine
        }
    }
}

fn get_native_this<'a>(env: &'a JNIEnv, obj: &JObject) -> Option<&'a Engine> {
    unsafe {
        let field: jlong = env.get_field_unchecked(*obj, CACHED_ABC_NATIVE_THIS_FIELD_ID.unwrap(), JavaType::Primitive(Primitive::Long)).unwrap().j().unwrap();
        if field != 0 {
            let eng = field as *mut Engine;
            return Some(&*eng);
        } else {
            return None;
        }
    }
}

fn get_string(env: &JNIEnv, jstr: &JString) -> String {
    let jstr_input = env.get_string(*jstr).expect("invalid string");
    let ptr = jstr_input.as_ptr();
    let c_str = unsafe { CStr::from_ptr(ptr) };
    match c_str.to_str() {
        Err(e) => panic!("Can not decode string {}", e),
        Ok(string) => return string.to_owned(),
    };
}


#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _: *mut c_void) -> jint {
    let env = vm.get_env().expect("Cannot get reference to the JNIEnv");
    unsafe {
        CACHED_ABC_NATIVE_THIS_FIELD_ID = Some(get_field_id(&env, "com/brave/adblock/AdBlockClient", "nativeThis", "J"));
    }
    JNI_VERSION_1_6
}

#[no_mangle]
pub extern fn Java_com_brave_adblock_AdBlockClient_deinit(env: JNIEnv, obj: JObject) {
    release_native_this_if_any(&env, &obj)
}

#[no_mangle]
pub extern fn Java_com_brave_adblock_AdBlockClient_loadRules(env: JNIEnv, obj: JObject, input: JString) -> jboolean {
    release_native_this_if_any(&env, &obj);

    let jstr_input = env.get_string(input).expect("invalid string");
    let ptr = jstr_input.as_ptr();
    let c_str = unsafe { CStr::from_ptr(ptr) };
    let rules: Vec<String> = match c_str.to_str() {
        Err(e) => {
            env.throw(format!("Can not decode string {}", e)).unwrap();
            return false as jboolean
        },
        Ok(string) => string.lines().map(|l| l.to_owned()).collect(),
    };

    let engine = Engine::from_rules(&rules, Default::default());
    let boxx = Box::new(engine);
    let jvalue = JValue::Long(Box::into_raw(boxx) as jlong);
    unsafe {
        env.set_field_unchecked(obj, CACHED_ABC_NATIVE_THIS_FIELD_ID.unwrap(), jvalue).unwrap();
    }
    true as jboolean
}

#[no_mangle]
pub extern fn Java_com_brave_adblock_AdBlockClient_serialize(env: JNIEnv, obj: JObject, fileName: JString) -> jboolean {
    if let Some(engine) = get_native_this(&env, &obj) {
        let serialized = engine.serialize_compressed().expect("Could not serialize!");
        let file_name = get_string(&env, &fileName);
        let mut file = File::create(file_name).expect("Could not create serialization file");
        file.write_all(&serialized).expect("Could not output serialized engine to file");
        true as jboolean
    } else {
        env.throw("No rules loaded!").unwrap();
        false as jboolean
    }
}

#[no_mangle]
pub extern fn Java_com_brave_adblock_AdBlockClient_deserialize(env: JNIEnv, obj: JObject, fileName: JString) -> jboolean {
    release_native_this_if_any(&env, &obj);
    let mut engine = Engine::default();
    let file_name = get_string(&env, &fileName);
    let mut file = File::open(file_name).unwrap();
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer).unwrap();
    let result = engine.deserialize(&buffer);
    if result.is_ok() {
        let boxx = Box::new(engine);
        let jvalue = JValue::Long(Box::into_raw(boxx) as jlong);
        unsafe {
            env.set_field_unchecked(obj, CACHED_ABC_NATIVE_THIS_FIELD_ID.unwrap(), jvalue).unwrap();
        }
    }
    result.is_ok() as jboolean
}

#[no_mangle]
pub extern fn Java_com_brave_adblock_AdBlockClient_matches(env: JNIEnv, obj: JObject, urlToCheck: JString, filterOption: jint, sourceUrl: JString) -> jboolean {
    if let Some(engine) = get_native_this(&env, &obj) {
        let url_to_check = get_string(&env, &urlToCheck);
        let source_url = get_string(&env, &sourceUrl);
        let result = engine.check_network_urls(&url_to_check, &source_url, REQUEST_TYPES[filterOption as usize]);
        return result.matched as jboolean;
    } else {
        env.throw("No rules loaded!").unwrap();
        false as jboolean
    }
}