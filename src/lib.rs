extern crate proc_macro;

use quote::quote;
use std::collections::HashMap;
use syn::Ident;

struct ItemStructs {
    ident_mapping: HashMap<Ident, Ident>,
    structs: Vec<syn::ItemStruct>,
}

impl syn::parse::Parse for ItemStructs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ident_mapping = HashMap::new();
        let mut structs = Vec::new();
        let ident_prefix = input.parse::<Ident>()?.to_string();
        let _ = input.parse::<syn::Token![,]>()?;

        while !input.is_empty() {
            let next_struct = input.parse::<syn::ItemStruct>()?;
            // TODO<24-07-19, Paul Lesur> unwrap!
            let orig_name = next_struct.ident.clone();
            let slim_name = Ident::new(
                &(ident_prefix.clone() + &orig_name.to_string()),
                proc_macro2::Span::call_site(),
            );
            ident_mapping.insert(orig_name, slim_name);
            structs.push(next_struct);
            input.is_empty();
        }
        Ok(ItemStructs {
            ident_mapping,
            structs,
        })
    }
}

impl ItemStructs {
    fn orig_structs(&self) -> &Vec<syn::ItemStruct> {
        &self.structs
    }

    fn rename_struct_type(&self, expr_struct: &mut syn::ItemStruct) {
        // TODO<24-07-19, Paul Lesur> unwrap!
        let orig_name = &expr_struct.ident;
        // TODO<24-07-19, Paul Lesur> unwrap!
        let slim_name = self.ident_mapping.get(&orig_name).unwrap();
        expr_struct.ident = slim_name.clone();
    }

    fn should_keep_field(f: &syn::Field) -> bool {
        for attr in f.attrs.iter() {
            if let syn::AttrStyle::Inner(_) = attr.style {
                continue;
            }

            let meta = attr.parse_meta().unwrap();

            match meta {
                syn::Meta::List(meta_list) => {
                    if meta_list.ident != "serde" {
                        continue;
                    }
                    for nested_meta in meta_list.nested {
                        match nested_meta {
                            syn::NestedMeta::Meta(syn::Meta::Word(word)) => {
                                if word == "skip_serializing" || word == "skip_deserializing" {
                                    return false;
                                }
                            }
                            // I *think* this is unreachable as we cannot nest another List here
                            // still, you never know...
                            syn::NestedMeta::Meta(_) => {
                                panic!("Apparently this is not unreachable, the more you know")
                            }
                            // a literal e.g "asd"
                            syn::NestedMeta::Literal(_) => {}
                        }
                    }
                }
                _ => continue,
            }
        }
        true
    }

    fn update_field_type(&self, mut field: syn::Field) -> syn::Field {
        match &mut field.ty {
            syn::Type::Path(type_path) => {
                let orig_name = &type_path.path.segments.last().unwrap().value().ident;
                if let Some(slim_name) = self.ident_mapping.get(&orig_name) {
                    type_path
                        .path
                        .segments
                        .last_mut()
                        .unwrap()
                        .value_mut()
                        .ident = slim_name.clone();
                }
            }
            _ => panic!(
                "Only types are supported in the struct definition;
                            but they do appear as 'syn::Expr::Path' when parsed through syn"
            ),
        }
        field
    }

    fn updated_field(&self, field: syn::Field) -> Option<syn::Field> {
        if ItemStructs::should_keep_field(&field) {
            Some(self.update_field_type(field))
        } else {
            None
        }
    }

    fn slim_structs(&self) -> Vec<syn::ItemStruct> {
        let mut v = Vec::new();

        for orig_struct in &self.structs {
            let mut slim_struct = orig_struct.clone();
            self.rename_struct_type(&mut slim_struct);
            slim_struct.fields = match slim_struct.fields {
                syn::Fields::Named(mut named) => {
                    named.named = named
                        .named
                        .into_iter()
                        .filter_map(|f| self.updated_field(f))
                        .collect();
                    named.into()
                }
                syn::Fields::Unnamed(mut unnamed) => {
                    unnamed.unnamed = unnamed
                        .unnamed
                        .into_iter()
                        .filter_map(|f| self.updated_field(f))
                        .collect();
                    unnamed.into()
                }
                syn::Fields::Unit => slim_struct.fields,
            };

            v.push(slim_struct);
        }

        v
    }
}

#[proc_macro]
pub fn serde_slim(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as ItemStructs);

    let mut output = quote! {};
    for orig_struct in input.orig_structs() {
        output = quote! { #orig_struct
            #output
        };
    }

    for slim_struct in input.slim_structs() {
        output = quote! { #slim_struct
            #output
        };
    }

    proc_macro::TokenStream::from(output)
}
