use jni::{objects::JObject, JNIEnv};
use ndk_sys as sys;
use std::{ffi::CString, ptr::NonNull};

const MODE_PRIVATE: i32 = 0;
const CONFIG_KEY: &str = "config";

pub fn load_asset(fname: &str) -> Vec<u8> {
    let android_context = ndk_context::android_context();

    let vm = unsafe { jni::JavaVM::from_raw(android_context.vm().cast()).unwrap() };
    let env = vm.attach_current_thread().unwrap();

    let asset_manager = unsafe {
        let jasset_manager = env
            .call_method(
                android_context.context().cast(),
                "getAssets",
                "()Landroid/content/res/AssetManager;",
                &[],
            )
            .unwrap()
            .l()
            .unwrap();
        let asset_manager_ptr =
            sys::AAssetManager_fromJava(env.get_native_interface(), jasset_manager.cast());

        ndk::asset::AssetManager::from_ptr(NonNull::new(asset_manager_ptr).unwrap())
    };

    let fname_cstring = CString::new(fname).unwrap();
    let mut asset = asset_manager.open(fname_cstring.as_c_str()).unwrap();
    asset.get_buffer().unwrap().to_vec()
}

fn get_preferences_object<'a>(env: &'a JNIEnv<'a>) -> JObject<'a> {
    let pref_name = env.new_string("alvr-pref").unwrap();

    env.call_method(
        ndk_context::android_context().context().cast(),
        "getSharedPreferences",
        "(Ljava/lang/String;I)Landroid/content/SharedPreferences;",
        &[pref_name.into(), MODE_PRIVATE.into()],
    )
    .unwrap()
    .l()
    .unwrap()
}

pub fn load_config_string() -> String {
    let android_context = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(android_context.vm().cast()).unwrap() };
    let env = vm.attach_current_thread().unwrap();

    let shared_preferences = get_preferences_object(&env);

    let key = env.new_string(CONFIG_KEY).unwrap();
    let default = env.new_string("").unwrap();

    let config = env
        .call_method(
            shared_preferences,
            "getString",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[key.into(), default.into()],
        )
        .unwrap();

    env.get_string(config.l().unwrap().into()).unwrap().into()
}

pub fn store_config_string(config: String) {
    let android_context = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(android_context.vm().cast()).unwrap() };
    let env = vm.attach_current_thread().unwrap();

    let shared_preferences = get_preferences_object(&env);

    let editor = env
        .call_method(
            shared_preferences,
            "edit",
            "()Landroid/content/SharedPreferences$Editor;",
            &[],
        )
        .unwrap()
        .l()
        .unwrap();

    let key = env.new_string(CONFIG_KEY).unwrap();
    let value = env.new_string(config).unwrap();
    env.call_method(
        editor,
        "putString",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;",
        &[key.into(), value.into()],
    )
    .unwrap();

    env.call_method(editor, "apply", "()V", &[]).unwrap();
}

pub fn device_name() -> String {
    let vm = unsafe { jni::JavaVM::from_raw(ndk_context::android_context().vm().cast()).unwrap() };
    let env = vm.attach_current_thread().unwrap();

    let jbrand_name = env
        .get_static_field("android/os/Build", "BRAND", "Ljava/lang/String;")
        .unwrap()
        .l()
        .unwrap();
    let brand_name_raw = env.get_string(jbrand_name.into()).unwrap();
    let brand_name = brand_name_raw.to_string_lossy().as_ref().to_owned();
    // Capitalize first letter
    let mut brand_name_it = brand_name.chars();
    let brand_name = brand_name_it
        .next()
        .unwrap()
        .to_uppercase()
        .chain(brand_name_it)
        .collect::<String>();

    let jdevice_name = env
        .get_static_field("android/os/Build", "MODEL", "Ljava/lang/String;")
        .unwrap()
        .l()
        .unwrap();
    let device_name_raw = env.get_string(jdevice_name.into()).unwrap();
    let device_name = device_name_raw.to_string_lossy().as_ref().to_owned();

    format!("{brand_name} {device_name}")
}
