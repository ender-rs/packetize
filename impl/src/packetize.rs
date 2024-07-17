use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, Ident, Index, Item, ItemStruct};

pub(crate) fn encode_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    match item {
        Item::Enum(value) => {
            let item_name = &value.ident;
            quote! {
                impl packetize::Encode for #item_name {
                    fn encode<const N: usize>
                        (&self, write_cursor: &mut fast_collections::Cursor<u8, N>) -> core::result::Result<(), ()> {
                        fast_collections::PushTransmute::push_transmute(write_cursor, Clone::clone(self))
                    }
                }
            }
        }
        Item::Struct(item_struct) => {
            let item_name = &item_struct.ident;
            let has_field_name = item_struct.fields.iter().last().map(|field| field.ident.is_some());
            let encode_constructor = generate_encoder(&item_struct, has_field_name);
            quote! {
               impl packetize::Encode for #item_name {
                   fn encode<const N: usize>
                       (&self, write_cursor: &mut fast_collections::Cursor<u8, N>) -> core::result::Result<(), ()> {
                       #encode_constructor
                       Ok(())
                   }
               }
            }
        },
        _ => panic!("unimplemented item type"),
    }
    .into()
}

pub(crate) fn decode_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    match item {
        Item::Enum(value) => {
            let item_name = &value.ident;
            quote! {
                impl packetize::Decode for #item_name {
                    fn decode<const N: usize>
                        (read_cursor: &mut fast_collections::cursor::Cursor<u8, N>) -> core::result::Result<Self, ()> {
                        fast_collections::CursorReadTransmute::read_transmute(read_cursor)
                            .map(|v| *v)
                            .ok_or_else(|| ())
                    }
                }
            }
        }
        Item::Struct(item_struct) => {
            let item_name = &item_struct.ident;
            let has_field_name = item_struct.fields.iter().last().map(|field| field.ident.is_some());
            let decode_constructor = generate_decoder(&item_struct, has_field_name);
            quote! {
               impl packetize::Decode for #item_name
               {
                   fn decode<const N: usize>
                       (read_cursor: &mut fast_collections::cursor::Cursor<u8, N>) -> Result<Self, ()> {
                       Ok(#decode_constructor)
                   }
               }
            }
        },
        _ => panic!("unimplemented item type"),
    }
    .into()
}

fn generate_decoder(
    item_struct: &ItemStruct,
    has_field_name: Option<bool>,
) -> proc_macro2::TokenStream {
    let decode = quote!(packetize::Decode::decode(read_cursor)?);
    if let Some(has_field_name) = has_field_name {
        if has_field_name {
            let fields: Vec<_> = item_struct
                .fields
                .iter()
                .map(|field| field.ident.clone().unwrap())
                .collect();
            quote! {
                Self {
                    #(#fields: #decode,)*
                }
            }
        } else {
            let fields: Vec<_> = (0..item_struct.fields.len())
                .map(|_| decode.clone())
                .collect();
            quote! {
                Self(
                    #(#fields,)*
                )
            }
        }
    } else {
        quote! {Self {}}
    }
}

fn generate_encoder(
    item_struct: &ItemStruct,
    has_field_name: Option<bool>,
) -> Option<proc_macro2::TokenStream> {
    if let Some(has_field_name) = has_field_name {
        Some(if has_field_name {
            let fields = map_fields_to_idents(&item_struct.fields);
            quote! {
                #(packetize::Encode::encode(&self.#fields, write_cursor)?;)*
            }
        } else {
            let fields = (0..item_struct.fields.len()).map(|i| Index::from(i));
            quote! {
                #(packetize::Encode::encode(&self.#fields, write_cursor)?;)*
            }
        })
    } else {
        None
    }
}

fn map_fields_to_idents(fields: &Fields) -> Vec<Ident> {
    fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect()
}
