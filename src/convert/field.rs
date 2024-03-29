use std::marker::PhantomData;
use std::str::FromStr;

use std::convert::{TryFrom, TryInto};

use jni::errors::Error as JniError;
use jni::errors::Result as JniResult;
use jni::objects::{JFieldID, JObject, JValueOwned};
use jni::signature::ReturnType;
use jni::JNIEnv;

use crate::convert::{FromJavaValue, IntoJavaValue, JavaValue, Signature, TryFromJavaValue, TryIntoJavaValue, JValueOwnedWrapper};
use crate::jni::objects::JValue;

pub struct Field<'env: 'borrow, 'borrow, T>
where
    T: Signature,
{
    env: &'borrow mut JNIEnv<'env>,
    field_id: JFieldID,
    obj: JObject<'env>,
    marker: PhantomData<T>,
}

impl<'env: 'borrow, 'borrow, T> Field<'env, 'borrow, T>
where
    T: Signature,
{
    pub fn new(
        env: &'borrow mut JNIEnv<'env>,
        obj: JObject<'env>,
        classpath_path: &str,
        field_name: &str,
    ) -> Option<Self> {
        let field_id = env
            .get_field_id(classpath_path, field_name, <T as Signature>::SIG_TYPE)
            .ok()?;

        Some(Field {
            env,
            field_id,
            obj,
            marker: Default::default(),
        })
    }
}

impl<'env: 'borrow, 'borrow, T> Field<'env, 'borrow, T>
where
    for<'inner_borrow> T: Signature + TryIntoJavaValue<'env> + TryFromJavaValue<'env, 'inner_borrow>,
    for<'inner_borrow> <T as TryFromJavaValue<'env, 'inner_borrow>>::Source: TryFrom<JValueOwnedWrapper<'env>, Error = JniError>,
    JValue<'env, 'env>: From<<T as TryIntoJavaValue<'env>>::Target>,
{
    pub fn set(&mut self, value: T) -> JniResult<()> {
        let v = TryIntoJavaValue::try_into(value, self.env)?;
        let jvalue: JValue = JValue::from(v);

        self.env
            .set_field_unchecked(&self.obj, self.field_id, jvalue)?;
        Ok(())
    }

    pub fn get(&mut self) -> JniResult<T> {
        let res: JValueOwned = self.env.get_field_unchecked(
            &self.obj,
            self.field_id,
            ReturnType::from_str(<T as Signature>::SIG_TYPE).unwrap(),
        )?;

        let f = JValueOwnedWrapper::from(res);
        TryInto::try_into(f).and_then(|v| TryFromJavaValue::try_from(v, self.env))
    }

    // Java object is not sufficient to retrieve parent object / field owner
    // We can use the owner as the source instead, but we don't have neither the field name nor the class classpath path
    // Don't implement this and use `#[field]` attribute instead?
    // A nicer solution would be to have a `const CLASS_PATH: &str` and a `const FIELD_NAME: &str` const parameters and use those instead,
    // but full const generics are required for that.
    // FIXME: use const generics to parametrize `Field` by class path and field name, and implement `(Try)FromJavaValue`
    pub fn field_try_from(
        source: JObject<'env>,
        classpath_path: &str,
        field_name: &str,
        env: &'borrow mut JNIEnv<'env>,
    ) -> JniResult<Self> {
        let class = env.find_class(classpath_path)?;
        let field_id = env.get_field_id(class, field_name, <T as Signature>::SIG_TYPE)?;

        let obj = source.autobox(env);
        Ok(Self {
            env,
            field_id,
            obj,
            marker: Default::default(),
        })
    }
}

impl<'env: 'borrow, 'borrow, T> Field<'env, 'borrow, T>
where
    for<'inner_borrow> T: Signature + IntoJavaValue<'env> + FromJavaValue<'env, 'inner_borrow>,
    for<'inner_borrow> <T as FromJavaValue<'env, 'inner_borrow>>::Source: TryFrom<JValueOwnedWrapper<'env>, Error = JniError>,
    JValue<'env, 'env>: From<<T as IntoJavaValue<'env>>::Target>,
{
    pub fn set_unchecked(&mut self, value: T) {
        let v = IntoJavaValue::into(value, self.env);
        let jvalue = JValue::from(v);

        self.env
            .set_field_unchecked(&self.obj, self.field_id, jvalue)
            .unwrap();
    }

    pub fn get_unchecked(&mut self) -> T {
        let res = self
            .env
            .get_field_unchecked(
                &self.obj,
                self.field_id,
                ReturnType::from_str(<T as Signature>::SIG_TYPE).unwrap(),
            )
            .unwrap();

        TryInto::try_into(JValueOwnedWrapper::from(res))
            .map(|v| FromJavaValue::from(v, self.env))
            .unwrap()
    }

    pub fn field_from(
        source: JObject<'env>,
        classpath_path: &str,
        field_name: &str,
        env: &'borrow mut JNIEnv<'env>,
    ) -> Self {
        let class = env.find_class(classpath_path).unwrap();
        let field_id = env
            .get_field_id(class, field_name, <T as Signature>::SIG_TYPE)
            .unwrap();

        let obj = source.autobox(env);
        Self {
            env,
            field_id,
            obj,
            marker: Default::default(),
        }
    }
}

impl<'env: 'borrow, 'borrow, T> Signature for Field<'env, 'borrow, T>
where
    T: Signature,
{
    const SIG_TYPE: &'static str = <T as Signature>::SIG_TYPE;
}
