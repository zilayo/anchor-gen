use std::collections::BTreeMap;

use anchor_syn::hash::hash;
use anchor_syn::idl::{IdlEvent, IdlField, IdlTypeDefinition};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{generate_struct, StructOpts};

/// Generates event structs.
pub fn generate_events(
    events: Option<&[IdlEvent]>,
    typedefs: &[IdlTypeDefinition],
    struct_opts: &BTreeMap<String, StructOpts>,
) -> TokenStream {
    match events {
        Some(events) => {
            let defined = events.iter().map(|def| {
                let struct_name = format_ident!("{}", def.name);
                let opts = struct_opts.get(&def.name).copied().unwrap_or_default();

                let discriminator: proc_macro2::TokenStream = {
                    let discriminator_preimage = format!("event:{}", struct_name).into_bytes();
                    let discriminator = hash(&discriminator_preimage);
                    format!("{:?}", &discriminator.0[..8]).parse().unwrap()
                };

                let fields = def
                    .fields
                    .iter()
                    .map(|f| IdlField {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                    })
                    .collect::<Vec<_>>();

                let struct_ts = generate_struct(typedefs, &struct_name, &fields, opts);

                quote! {
                    #struct_ts

                    impl anchor_lang::Discriminator for #struct_name {
                        const DISCRIMINATOR: [u8; 8] = #discriminator;
                        fn discriminator() -> [u8; 8] {
                          self::DISCRIMINATOR
                        }
                    }

                    impl anchor_lang::Event for #struct_name {
                      fn data(&self) -> Vec<u8> {
                          let mut data = Vec::with_capacity(256);
                          data.extend_from_slice(&#discriminator);
                          self.serialize(&mut data).unwrap();
                          data
                      }
                  }
                }
            });
            quote! {
                #(#defined)*
            }
        }
        None => quote!(),
    }
}
