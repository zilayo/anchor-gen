use std::collections::BTreeMap;

use anchor_syn::idl::{IdlEvent, IdlField, IdlTypeDefinition};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use sha2::{Digest, Sha256};

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
                    let discriminator_preimage = format!("event:{}", struct_name);
                    let mut hash = Sha256::new();
                    hash.update(discriminator_preimage.as_bytes());
                    let discriminator_bytes = &hash.finalize()[..8];
                    let discriminator_vec = discriminator_bytes
                        .iter()
                        .map(|byte| quote!(#byte))
                        .collect::<Vec<_>>();
                    quote!([#(#discriminator_vec),*])
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
                            #discriminator
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
