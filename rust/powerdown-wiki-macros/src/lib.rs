use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Type};

#[proc_macro_derive(UserModified)]
/// Given a struct, converts all of its properties into Options such that serde will handle it as follows:
/// 
/// If there's a struct, X: i32, Y: Option<i32>, Z: Option<i32>
/// And serde reads a json of {X:3, Y:2}
/// The result will be X: Some(3), Y:Some(Some(2)), Z: None
/// 
/// Meant to be used when users send modifications to items on the Powerdown Wiki
pub fn user_modified_derive(input: TokenStream) -> TokenStream {
    todo!("Unimplemented")
}

