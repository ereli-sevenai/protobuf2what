use crate::parser::ast::{Enum, Field, Message, Method, ProtoFile, ProtoOption, Service};

/// Visitor trait for traversing the Protocol Buffer AST
///
/// This trait defines methods for visiting each node type in the Protocol Buffer AST.
/// Implementations should override these methods to perform specific operations on each node type.
pub trait Visitor {
    /// Visit a Protocol Buffer file
    fn visit_proto_file(&mut self, proto_file: &ProtoFile) {
        // Visit all options
        for option in &proto_file.options {
            self.visit_option(option);
        }
        
        // Visit all messages
        for message in &proto_file.messages {
            self.visit_message(message);
        }
        
        // Visit all enums
        for enum_def in &proto_file.enums {
            self.visit_enum(enum_def);
        }
        
        // Visit all services
        for service in &proto_file.services {
            self.visit_service(service);
        }
    }
    
    /// Visit a message
    fn visit_message(&mut self, message: &Message) {
        // Visit message options
        for option in &message.options {
            self.visit_option(option);
        }
        
        // Visit all fields
        for field in &message.fields {
            self.visit_field(field);
        }
        
        // Visit nested messages
        for nested_message in &message.nested_messages {
            self.visit_message(nested_message);
        }
        
        // Visit nested enums
        for nested_enum in &message.nested_enums {
            self.visit_enum(nested_enum);
        }
    }
    
    /// Visit an enum
    fn visit_enum(&mut self, enum_def: &Enum) {
        // Visit enum options
        for option in &enum_def.options {
            self.visit_enum_value_option(option);
        }
        
        // Visit all enum values
        for value in &enum_def.values {
            self.visit_enum_value(value);
        }
    }
    
    /// Visit a service
    fn visit_service(&mut self, service: &Service) {
        // Visit service options
        for option in &service.options {
            self.visit_option(option);
        }
        
        // Visit all methods
        for method in &service.methods {
            self.visit_method(method);
        }
    }
    
    /// Visit a field
    fn visit_field(&mut self, field: &Field) {
        // Visit field options
        for option in &field.options {
            self.visit_option(option);
        }
    }
    
    /// Visit an enum value
    fn visit_enum_value(&mut self, enum_value: &crate::parser::ast::EnumValue) {
        // Visit enum value options
        for option in &enum_value.options {
            self.visit_enum_value_option(option);
        }
    }
    
    /// Visit a method
    fn visit_method(&mut self, method: &Method) {
        // Visit method options
        for option in &method.options {
            self.visit_option(option);
        }
    }
    
    /// Visit an option
    fn visit_option(&mut self, _option: &ProtoOption) {
        // Default implementation does nothing
    }
    
    /// Visit an enum value option
    fn visit_enum_value_option(&mut self, _option: &crate::parser::ast::EnumValueOption) {
        // Default implementation does nothing
    }
}
