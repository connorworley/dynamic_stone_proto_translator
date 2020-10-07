#![feature(trait_alias)]

use std::collections::HashMap;

use protobuf::Message;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let file_descriptor_set = protobuf::descriptor::FileDescriptorSet::parse_from_bytes(
        include_bytes!("fixture.descriptor"),
    )?;
    // convert from a vector of FileDescriptorProtos to FileDescriptors
    let file_descriptors = protobuf::reflect::FileDescriptor::new_dynamic_fds(file_descriptor_set.file);
    let unmarshalers = file_descriptors.into_iter().map(
        |file_descriptor| (file_descriptor.proto().get_name().to_owned(), generate_unmarshalers(file_descriptor))
    ).collect::<HashMap<_, _>>();
    unmarshalers["fixture.proto"]["MessageFixture"]("json data would go here");
    Ok(())
}

fn generate_unmarshalers(file_descriptor: protobuf::reflect::FileDescriptor) -> HashMap<String, Box<impl Fn(&str) -> Box<dyn protobuf::MessageDyn>>> {
    file_descriptor.messages().into_iter().map(
        |message_descriptor| (message_descriptor.get_name().to_owned(), Box::new(generate_message_unmarshaler(message_descriptor)))
    ).collect()
}

fn generate_message_unmarshaler(message_descriptor: protobuf::reflect::MessageDescriptor) -> impl Fn(&str) -> Box<dyn protobuf::MessageDyn> {
    move |json| {
        // TODO: parse json
        let mut msg = message_descriptor.new_instance();
        let msg_fields = msg.mut_unknown_fields_dyn();
        for field in message_descriptor.fields() {
            // TODO: fill msg with data from parsed json
            // Example: field.set_singular_field(&mut *msg, protobuf::reflect::ReflectValueBox::I32(1234));
            // exact invocation depends on field type.
            println!(
                "{:?}: {:?} {:?} (repeated={:?}, map={:?})",
                field.get_name(),
                field.get_proto().get_field_type(),
                field.get_proto().get_type_name(),
                field.is_repeated(),
                field.is_map(),
            );
            // Output:
            // "number": TYPE_INT32 "" (repeated=false, map=false)
            // "text": TYPE_STRING "" (repeated=false, map=false)
            // "repeated_number": TYPE_INT32 "" (repeated=true, map=false)
            // "msg": TYPE_MESSAGE ".fixture.MessageFixture.Nested" (repeated=false, map=false)
        }
        msg
    }
}
