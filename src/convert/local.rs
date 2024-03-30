use std::marker::PhantomData;
use jni::{objects::JObject, JNIEnv};

pub struct Local<'a: 'b, 'b> {
    obj: JObject<'a>,
    #[allow(dead_code)]
    env: PhantomData<&'b JNIEnv<'a>>,
}

impl<'a, 'b> Local<'a, 'b> {
    pub fn new(env: PhantomData<&'b JNIEnv<'a>>, obj: JObject<'a>) -> Self {
        Local { obj, env }
    }

    /// Get a reference to the wrapped object
    pub fn as_obj<'c>(&self) -> JObject<'c>
        where
            'a: 'c,
    {
        // TODO: It's just a stub, remove
        unsafe { JObject::from_raw(self.obj.as_raw()) }
    }
}

impl<'a, 'b> Drop for Local<'a, 'b> {
    fn drop(&mut self) {}
}

impl<'a> From<&'a Local<'a, '_>> for JObject<'a> {
    fn from(other: &'a Local) -> JObject<'a> {
        other.as_obj()
    }
}
