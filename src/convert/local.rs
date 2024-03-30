use std::marker::PhantomData;
use jni::{objects::JObject, JNIEnv};

pub struct Local<'a: 'b, 'b> {
    obj: JObject<'a>,
    // Todo: Remove if Field doesn't require a second lifetime
    // Added for simplicity, to let bridge struct have two lifetime params
    // both with and without Field members
    env: PhantomData<&'b ()>,
}

impl<'a, 'b> Local<'a, 'b> {
    pub fn new(env: PhantomData<&'b ()>, obj: JObject<'a>) -> Self {
        Local { obj, env }
    }

    /// Get a reference to the wrapped object
    pub fn into_obj<'c>(self) -> JObject<'c>
        where
            'a: 'c,
    {
        self.obj
    }

    pub fn as_obj<'c, 'd>(&'d self) -> &'d JObject<'c>
        where
            'a: 'c,
            'b: 'd,
    {
        &self.obj
    }
}

// impl<'a> From<Local<'a, '_>> for JObject<'a> {
//     fn from(other: Local<'a, '_>) -> JObject<'a> {
//         other.into_obj()
//     }
// }
