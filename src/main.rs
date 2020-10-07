#![feature(type_alias_impl_trait)]

mod fixture;

use protobuf::Message;
use std::collections::HashMap;

type Unmarshaller =
    impl Fn(
        &serde_json::Value,
    ) -> Result<Box<dyn protobuf::MessageDyn>, Box<dyn std::error::Error + 'static>>;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let file_descriptor_set = protobuf::descriptor::FileDescriptorSet::parse_from_bytes(
        include_bytes!("fixture.descriptor"),
    )?;
    // convert from a vector of FileDescriptorProtos to FileDescriptors
    let file_descriptors =
        protobuf::reflect::FileDescriptor::new_dynamic_fds(file_descriptor_set.file);
    let unmarshallers = file_descriptors
        .into_iter()
        .map(|file_descriptor| {
            (
                file_descriptor.proto().get_name().to_owned(),
                generate_unmarshallers(file_descriptor),
            )
        })
        .collect::<HashMap<_, _>>();
    let dynamic_message = unmarshallers["fixture.proto"]["MessageFixture"](&serde_json::from_str(
        r#"{"number": 1, "text": "hello", "repeated_number": [1, 2, 3], "msg": {"foo": 123}}"#,
    )?)?;
    println!(
        "{}",
        protobuf::json::print_to_string(dynamic_message.as_ref())
            .or(Err("failed to format message as json"))?
    );
    Ok(())
}

fn generate_unmarshallers(
    file_descriptor: protobuf::reflect::FileDescriptor,
) -> HashMap<String, Box<Unmarshaller>> {
    file_descriptor
        .messages()
        .into_iter()
        .map(|message_descriptor| {
            (
                message_descriptor.get_name().to_owned(),
                Box::new(generate_message_unmarshaller(message_descriptor)),
            )
        })
        .collect()
}

fn generate_message_unmarshaller(
    message_descriptor: protobuf::reflect::MessageDescriptor,
) -> Unmarshaller {
    move |json| {
        let mut msg = message_descriptor.new_instance();
        for field in message_descriptor.fields() {
            match field.runtime_field_type() {
                protobuf::reflect::RuntimeFieldType::Singular(t) => {
                    field.set_singular_field(
                        &mut *msg,
                        value_from_json(json.get(field.get_name()).ok_or("field missing")?, &t)?,
                    );
                }
                protobuf::reflect::RuntimeFieldType::Repeated(t) => {
                    let arr = json
                        .get(field.get_name())
                        .ok_or("field missing")?
                        .as_array()
                        .ok_or("could not read array")?;
                    let mut repeated = field.mut_repeated(&mut *msg);
                    for element in arr {
                        repeated.push(value_from_json(element, &t)?);
                    }
                }
                protobuf::reflect::RuntimeFieldType::Map(_, _) => {
                    return Err("map not supported".into())
                }
            }
        }
        Ok(msg)
    }
}

fn value_from_json(
    json: &serde_json::Value,
    t: &protobuf::reflect::RuntimeTypeBox,
) -> Result<protobuf::reflect::ReflectValueBox, Box<dyn std::error::Error + 'static>> {
    Ok(match t {
        protobuf::reflect::RuntimeTypeBox::I32 => protobuf::reflect::ReflectValueBox::I32(
            json.as_i64().ok_or("could not read i64")? as i32,
        ),
        protobuf::reflect::RuntimeTypeBox::I64 => {
            protobuf::reflect::ReflectValueBox::I64(json.as_i64().ok_or("could not read i64")?)
        }
        protobuf::reflect::RuntimeTypeBox::U32 => protobuf::reflect::ReflectValueBox::U32(
            json.as_u64().ok_or("could not read u64")? as u32,
        ),
        protobuf::reflect::RuntimeTypeBox::U64 => {
            protobuf::reflect::ReflectValueBox::U64(json.as_u64().ok_or("could not read u64")?)
        }
        protobuf::reflect::RuntimeTypeBox::F32 => protobuf::reflect::ReflectValueBox::F32(
            json.as_f64().ok_or("could not read f64")? as f32,
        ),
        protobuf::reflect::RuntimeTypeBox::F64 => {
            protobuf::reflect::ReflectValueBox::F64(json.as_f64().ok_or("could not read f64")?)
        }
        protobuf::reflect::RuntimeTypeBox::Bool => {
            protobuf::reflect::ReflectValueBox::Bool(json.as_bool().ok_or("could not read bool")?)
        }
        protobuf::reflect::RuntimeTypeBox::String => protobuf::reflect::ReflectValueBox::String(
            json.as_str().ok_or("could not read string")?.to_string(),
        ),
        protobuf::reflect::RuntimeTypeBox::VecU8 => return Err("bytes not supported".into()),
        protobuf::reflect::RuntimeTypeBox::Enum(_) => return Err("enum not supported".into()),
        protobuf::reflect::RuntimeTypeBox::Message(message_descriptor) => {
            protobuf::reflect::ReflectValueBox::Message(generate_message_unmarshaller(
                message_descriptor.clone(),
            )(json)?)
        }
    })
}
