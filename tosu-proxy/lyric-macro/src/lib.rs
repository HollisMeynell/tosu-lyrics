extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Lit, parse_macro_input};

#[proc_macro_derive(Setting, attributes(default))]
pub fn setting_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;
    let vis = &input.vis;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named_fields) => &named_fields.named,
            _ => panic!("不支持匿名类型, 必须要有名字作为 `key`"),
        },
        _ => panic!("宏只支持在 `struct` 使用"),
    };

    let enum_name = Ident::new(&format!("{}Type", struct_name), struct_name.span());

    let enum_variants = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_type = &f.ty;
        let variant_name = Ident::new(
            &field_name.to_string().to_case(Case::UpperCamel),
            field_name.span(),
        );
        quote! {
            #variant_name(#field_type)
        }
    });

    let init_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_type = &f.ty;
        let field_name_str = field_name.to_string();
        let default_expr = get_default_attr(f).unwrap_or_else(|| {
            quote! { #field_type::default() }
        });
        quote! {
            let #field_name = match SettingEntity::get_config(#field_name_str).await {
                None => { #default_expr }
                Some(value) => { serde_json::from_str(&value).unwrap_or_default() }
            };
        }
    });

    let struct_init_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        quote! { #field_name }
    });

    let methods = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_type = &f.ty;
        let setter_name = Ident::new(&format!("set_{}", field_name), field_name.span());
        let getter_name = Ident::new(&format!("get_{}", field_name), field_name.span());
        let field_name_str = field_name.to_string();

        quote! {
            #vis fn #setter_name(&mut self, value: #field_type) {
                let value_str = serde_json::to_string(&value).unwrap();
                tokio::spawn(async {
                    SettingEntity::save_config(String::from(#field_name_str), value_str).await;
                });
                self.#field_name = value;
            }

            #vis fn #getter_name(&self) -> &#field_type {
                &self.#field_name
            }
        }
    });

    let get_method_match_arms = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let variant_name = Ident::new(
            &field_name.to_string().to_case(Case::UpperCamel),
            field_name.span(),
        );
        quote! {
            #field_name_str => Some(#enum_name::#variant_name(self.#field_name.clone())),
        }
    });

    let expanded = quote! {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        #vis enum #enum_name {
            #(#enum_variants),*
        }

        impl #struct_name {
            #vis async fn init() -> Self {
                #(#init_fields)*

                #struct_name {
                    #(#struct_init_fields),*
                }
            }

            #(#methods)*

            #vis fn get(&self, key: &str) -> Option<#enum_name> {
                match key {
                    #(#get_method_match_arms)*
                    _ => None,
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_default_attr(field: &syn::Field) -> Option<proc_macro2::TokenStream> {
    for attr in &field.attrs {
        if !attr.path().is_ident("default") {
            continue;
        }

        let lit = match attr.parse_args::<Lit>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        match lit {
            Lit::Str(s) => {
                return Some(quote! { #s.to_string() });
            }
            Lit::ByteStr(s) => {
                return Some(quote! { #s });
            }
            Lit::Byte(b) => {
                return Some(quote! { #b });
            }
            Lit::Char(c) => {
                return Some(quote! { #c });
            }
            Lit::Bool(b) => {
                let value = b.value;
                return Some(quote! { #value });
            }
            Lit::Int(i) => {
                return Some(quote! { #i });
            }
            Lit::Float(f) => {
                return Some(quote! { #f });
            }
            Lit::Verbatim(v) => {
                return Some(quote! { #v });
            }
            _ => {}
        }
    }
    None
}