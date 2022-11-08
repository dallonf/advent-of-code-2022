use crate::prelude::*;
use rlua::prelude::*;
use serde::{
    ser::{self, SerializeSeq, SerializeStruct, SerializeTuple, SerializeTupleStruct},
    Serialize, Serializer,
};

pub fn to_lua<'lua>(ctx: LuaContext<'lua>, value: impl Serialize) -> Result<LuaValue<'lua>> {
    let mut serializer = LuaSerializer::new(ctx);
    value.serialize(&mut serializer).map_err(Into::into)
}

pub struct LuaSerializer<'lua> {
    ctx: LuaContext<'lua>,
}

impl<'lua> LuaSerializer<'lua> {
    pub fn new(ctx: LuaContext<'lua>) -> Self {
        Self { ctx }
    }
}

#[derive(Error, Debug)]
pub enum LuaSerializeError {
    #[error("{0:?}")]
    Other(Error),
    #[error("{0:?}")]
    LuaError(LuaError),
    #[error("missing key trying to serialize a table field")]
    MissingKey,
}
impl serde::ser::Error for LuaSerializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        LuaSerializeError::Other(anyhow!(msg.to_string()))
    }
}
impl From<LuaError> for LuaSerializeError {
    fn from(err: LuaError) -> Self {
        Self::LuaError(err)
    }
}
impl From<Error> for LuaSerializeError {
    fn from(err: Error) -> Self {
        Self::Other(err)
    }
}

impl<'lua, 'a> Serializer for &'a mut LuaSerializer<'lua> {
    type Ok = LuaValue<'lua>;
    type Error = LuaSerializeError;

    type SerializeSeq = LuaListSerializer<'lua>;
    type SerializeTuple = LuaListSerializer<'lua>;
    type SerializeTupleStruct = LuaListSerializer<'lua>;
    type SerializeTupleVariant = LuaVariantSerializer<'lua, LuaListSerializer<'lua>>;
    type SerializeMap = LuaTableSerializer<'lua>;
    type SerializeStruct = LuaTableSerializer<'lua>;
    type SerializeStructVariant = LuaVariantSerializer<'lua, LuaTableSerializer<'lua>>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(LuaValue::Boolean(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(LuaValue::Number(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(self
            .ctx
            .create_string(v.as_bytes())
            .map_err(|err| LuaSerializeError::Other(anyhow!(err)))?
            .to_lua(self.ctx)?)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut serializer = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            SerializeSeq::serialize_element(&mut serializer, byte)?;
        }
        SerializeSeq::end(serializer)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(LuaNil)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let mut serializer = LuaTableSerializer::new(self.ctx)?;
        serializer.serialize_field("type", variant)?;
        serializer.serialize_field("value", value)?;
        serializer.end()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        LuaListSerializer::new(self.ctx).map_err(Into::into)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        LuaListSerializer::new(self.ctx).map_err(Into::into)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        LuaListSerializer::new(self.ctx).map_err(Into::into)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        LuaVariantSerializer::new(self.ctx, variant, LuaListSerializer::new(self.ctx)?)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        LuaTableSerializer::new(self.ctx).map_err(Into::into)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        LuaTableSerializer::new(self.ctx).map_err(Into::into)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        LuaVariantSerializer::new(self.ctx, variant, LuaTableSerializer::new(self.ctx)?)
    }
}

pub struct LuaListSerializer<'lua> {
    ctx: LuaContext<'lua>,
    current_value: LuaTable<'lua>,
}
impl<'lua> LuaListSerializer<'lua> {
    pub fn new(ctx: LuaContext<'lua>) -> Result<Self> {
        Ok(Self {
            ctx,
            current_value: ctx.create_table()?,
        })
    }
}
impl<'lua> ser::SerializeSeq for LuaListSerializer<'lua> {
    type Ok = LuaValue<'lua>;
    type Error = LuaSerializeError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let index = self.current_value.len()?;
        self.current_value.set(
            index + 1,
            value.serialize(&mut LuaSerializer::new(self.ctx))?,
        )?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.current_value.to_lua(self.ctx).map_err(Into::into)
    }
}

impl<'lua> ser::SerializeTuple for LuaListSerializer<'lua> {
    type Ok = LuaValue<'lua>;
    type Error = LuaSerializeError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'lua> ser::SerializeTupleStruct for LuaListSerializer<'lua> {
    type Ok = LuaValue<'lua>;
    type Error = LuaSerializeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

pub struct LuaTableSerializer<'lua> {
    ctx: LuaContext<'lua>,
    current_value: LuaTable<'lua>,
    current_key: Option<LuaValue<'lua>>,
}
impl<'lua> LuaTableSerializer<'lua> {
    pub fn new(ctx: LuaContext<'lua>) -> Result<Self> {
        Ok(Self {
            ctx,
            current_value: ctx.create_table()?,
            current_key: None,
        })
    }
}

impl<'lua> ser::SerializeMap for LuaTableSerializer<'lua> {
    type Ok = LuaValue<'lua>;
    type Error = LuaSerializeError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.current_key = Some(key.serialize(&mut LuaSerializer::new(self.ctx))?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let key = self
            .current_key
            .take()
            .ok_or(LuaSerializeError::MissingKey)?;
        self.current_value
            .set(key, value.serialize(&mut LuaSerializer::new(self.ctx))?)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.current_value.to_lua(self.ctx).map_err(Into::into)
    }
}

impl<'lua> ser::SerializeStruct for LuaTableSerializer<'lua> {
    type Ok = LuaValue<'lua>;
    type Error = LuaSerializeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeMap::end(self)
    }
}

pub struct LuaVariantSerializer<'lua, T> {
    ctx: LuaContext<'lua>,
    current_value: LuaTable<'lua>,
    sub_serializer: T,
}
impl<'lua, T> LuaVariantSerializer<'lua, T> {
    pub fn new(
        ctx: LuaContext<'lua>,
        type_name: &str,
        sub_serializer: T,
    ) -> std::result::Result<Self, LuaSerializeError> {
        let table = ctx.create_table()?;
        table.set("type", type_name)?;
        Ok(Self {
            ctx,
            current_value: table,
            sub_serializer,
        })
    }
}
impl<'lua> ser::SerializeStructVariant for LuaVariantSerializer<'lua, LuaTableSerializer<'lua>> {
    type Ok = LuaValue<'lua>;
    type Error = LuaSerializeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.sub_serializer.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let value = self.sub_serializer.end()?;
        self.current_value.set("value", value)?;
        self.current_value.to_lua(self.ctx).map_err(Into::into)
    }
}
impl<'lua> ser::SerializeTupleVariant for LuaVariantSerializer<'lua, LuaListSerializer<'lua>> {
    type Ok = LuaValue<'lua>;
    type Error = LuaSerializeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.sub_serializer.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let value = SerializeTuple::end(self.sub_serializer)?;
        self.current_value.set("value", value)?;
        self.current_value.to_lua(self.ctx).map_err(Into::into)
    }
}
