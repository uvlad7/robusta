//! Fallible conversions traits.
//!
//! These are the traits selected if `call_type` is omitted or if specified with a `safe` parameter.
//!
//! ```ignore
//! #[call_type(safe)]
//! ```
//!
//! If any conversion fails, e.g. while converting input parameters or return arguments a Java exception is thrown.
//! Exception class and exception message can be customized with the `exception_class` and `message` parameters of the `safe` option, as such:
//!
//! ```ignore
//! #[call_type(safe(exception_class = "java.io.IOException", message = "Error while calling JNI function!"))]
//! ```
//!
//! Both of these parameters are optional. By default, the exception class is `java.lang.RuntimeException`.
//!

use jni::errors::{Result, Error};
use jni::objects::{JList, JObject, JString, JValue};
use jni::sys::{jboolean, jbooleanArray, jchar, jobject, jstring};
use jni::JNIEnv;

use crate::convert::unchecked::{FromJavaValue, IntoJavaValue};
use crate::convert::{JavaValue, Signature};

/// Conversion trait from Rust values to Java values, analogous to [TryInto](std::convert::TryInto). Used when converting types returned from JNI-available functions.
pub trait TryIntoJavaValue<'env>: Signature {
    type Target: JavaValue<'env>;
    const SIG_TYPE: &'static str = <Self as Signature>::SIG_TYPE;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target>;
}

/// Conversion trait from Java values to Rust values, analogous to [TryFrom](std::convert::TryInto). Used when converting types that are input to JNI-available functions.
pub trait TryFromJavaValue<'env: 'borrow, 'borrow>
where
    Self: Sized + Signature, {
    type Source: JavaValue<'env>;
    const SIG_TYPE: &'static str = <Self as Signature>::SIG_TYPE;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self>;
}

impl<'env, T> TryIntoJavaValue<'env> for T
where
    T: JavaValue<'env> + Signature,
{
    type Target = T;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        Ok(IntoJavaValue::into(self, env))
    }
}

impl<'env: 'borrow, 'borrow, T> TryFromJavaValue<'env, 'borrow> for T
where
    T: JavaValue<'env> + Signature,
{
    type Source = T;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        Ok(FromJavaValue::from(s, env))
    }
}

impl<'env> TryIntoJavaValue<'env> for String {
    type Target = jstring;
    const SIG_TYPE: &'static str = "Ljava/lang/String;";

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        env.new_string(self).map(|s| s.into_inner())
    }
}

impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for String {
    type Source = JString<'env>;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        env.get_string(s).map(Into::into)
    }
}

impl<'env> TryIntoJavaValue<'env> for bool {
    type Target = jboolean;

    fn try_into(self, _env: &JNIEnv<'env>) -> Result<Self::Target> {
        Ok(IntoJavaValue::into(self, _env))
    }
}

impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for bool {
    type Source = jboolean;

    fn try_from(s: Self::Source, _env: &JNIEnv<'env>) -> Result<Self> {
        Ok(FromJavaValue::from(s, _env))
    }
}

impl<'env> TryIntoJavaValue<'env> for char {
    type Target = jchar;

    fn try_into(self, _env: &JNIEnv<'env>) -> Result<Self::Target> {
        Ok(IntoJavaValue::into(self, _env))
    }
}

impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for char {
    type Source = jchar;

    fn try_from(s: Self::Source, _env: &JNIEnv<'env>) -> Result<Self> {
        let res = std::char::decode_utf16(std::iter::once(s))
            .next();

        match res {
            Some(Ok(c)) => Ok(c),
            Some(Err(_)) | None => Err(Error::from(format!("invalid `jchar` value encountered when casting to `char`: {}", s)))
        }
    }
}

impl Signature for Box<[bool]> {
    const SIG_TYPE: &'static str = "[Z";
}

impl<'env> TryIntoJavaValue<'env> for Box<[bool]> {
    type Target = jbooleanArray;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        let len = self.len();
        let buf: Vec<_> = self.iter().map(|&b| Into::into(b)).collect();
        let raw = env.new_boolean_array(len as i32)?;
        env.set_boolean_array_region(raw, 0, &buf)?;
        Ok(raw)
    }
}

impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for Box<[bool]> {
    type Source = jbooleanArray;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        let len = env.get_array_length(s)?;
        let mut buf = Vec::with_capacity(len as usize).into_boxed_slice();
        env.get_boolean_array_region(s, 0, &mut *buf)?;

        buf.iter()
            .map(|&b| TryFromJavaValue::try_from(b, &env))
            .collect()
    }
}

impl<'env, T> TryIntoJavaValue<'env> for Vec<T>
where
    T: TryIntoJavaValue<'env>,
{
    type Target = jobject;

    fn try_into(self, env: &JNIEnv<'env>) -> Result<Self::Target> {
        let obj = env.new_object(
            "java/util/ArrayList",
            "(I)V",
            &[JValue::Int(self.len() as i32)],
        )?;
        let list = JList::from_env(&env, obj)?;

        let _: Result<Vec<_>> = self
            .into_iter()
            .map::<Result<_>, _>(|el| {
                Ok(JavaValue::autobox(
                    TryIntoJavaValue::try_into(el, &env)?,
                    &env,
                ))
            })
            .map(|el| Ok(list.add(el?)))
            .collect();

        Ok(list.into_inner())
    }
}

impl<'env: 'borrow, 'borrow, T, U> TryFromJavaValue<'env, 'borrow> for Vec<T>
where
    T: TryFromJavaValue<'env, 'borrow, Source = U>,
    U: JavaValue<'env>,
{
    type Source = JObject<'env>;

    fn try_from(s: Self::Source, env: &'borrow JNIEnv<'env>) -> Result<Self> {
        let list = JList::from_env(env, s)?;

        list.iter()?
            .map(|el| Ok(T::try_from(U::unbox(el, env), env)?))
            .collect()
    }
}
