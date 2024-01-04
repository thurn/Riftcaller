// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Macros used throughout the project

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, Fields, GenericArgument, Path,
    PathArguments, Type,
};

/// Generates Event & Query structs for the delegates in the delegate enum.
/// See the module comment in `delegate_data` for more information about this
/// system.
#[proc_macro_derive(GameDelegateEnum)]
pub fn game_delegate_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let tokens = game_implementation(&ast).unwrap_or_else(|err| err.to_compile_error());
    tokens.into()
}

fn game_implementation(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let parsed = parse(ast, GenerationMode::Game);
    generated(parsed?)
}

/// Equivalent macro to `GameDelegateEnum` for generating adventure-mode
/// delegates
#[proc_macro_derive(AdventureDelegateEnum)]
pub fn adventure_delegate_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let tokens = adventure_implementation(&ast).unwrap_or_else(|err| err.to_compile_error());
    tokens.into()
}

fn adventure_implementation(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let parsed = parse(ast, GenerationMode::Adventure);
    generated(parsed?)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GenerationMode {
    Game,
    Adventure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DelegateType {
    Event,
    Query,
}

#[derive(Debug, Clone)]
struct ParsedVariant {
    mode: GenerationMode,
    enum_name: Ident,
    name: Ident,
    docs: Vec<Attribute>,
    data: Path,
    output: Option<Path>,
    delegate_type: DelegateType,
}

fn parse(ast: &DeriveInput, mode: GenerationMode) -> syn::Result<Vec<ParsedVariant>> {
    let enum_name = &ast.ident;
    let Data::Enum(data) = &ast.data else {
        return Err(error("Expected enum"));
    };

    let mut result = vec![];
    for variant in &data.variants {
        let Fields::Unnamed(fields) = &variant.fields else {
            return Err(error("Expected unnamed field"));
        };

        let docs = variant
            .attrs
            .iter()
            .filter(|attribute| attribute.path.is_ident("doc"))
            .cloned()
            .collect();
        let field = fields.unnamed.iter().next().ok_or_else(|| error("Expected a field"))?;
        let Type::Path(path) = &field.ty else {
            return Err(error("Expected path"));
        };

        let segment =
            &path.path.segments.iter().next().ok_or_else(|| error("Expected a path segment"))?;
        let PathArguments::AngleBracketed(args) = &segment.arguments else {
            return Err(error("Expected PathArguments::AngleBracketed"));
        };

        let delegate_type = if segment.ident.to_string().contains("Query") {
            DelegateType::Query
        } else {
            DelegateType::Event
        };

        result.push(ParsedVariant {
            mode,
            enum_name: enum_name.clone(),
            name: variant.ident.clone(),
            docs,
            data: generic_argument(args, 0)?.clone(),
            output: if delegate_type == DelegateType::Event {
                None
            } else {
                Some(generic_argument(args, 1)?.clone())
            },
            delegate_type,
        });
    }

    Ok(result)
}

fn generated(variants: Vec<ParsedVariant>) -> syn::Result<TokenStream> {
    let variants = variants.iter().map(generate_variant);
    Ok(quote! {
        #(#variants)*
    })
}

fn generate_variant(variant: &ParsedVariant) -> impl ToTokens {
    let enum_name = &variant.enum_name;
    let name = &variant.name;
    let struct_name = format_ident!(
        "{}{}",
        name,
        if variant.delegate_type == DelegateType::Event { "Event" } else { "Query" }
    );
    let docs = &variant.docs;
    let data = &variant.data;

    let (trait_value, return_value) = match (variant.mode, variant.delegate_type) {
        (GenerationMode::Game, DelegateType::Event) => {
            (quote! {EventData<#data>}, quote! {Option<&EventDelegate<#data>>})
        }
        (GenerationMode::Game, DelegateType::Query) => {
            let output = variant.output.as_ref().expect("output");
            (quote! {QueryData<#data, #output>}, quote! {Option<&QueryDelegate<#data, #output>>})
        }
        (GenerationMode::Adventure, DelegateType::Event) => {
            (quote! {AdventureEventData<#data>}, quote! {Option<&AdventureEvent<#data>>})
        }
        (GenerationMode::Adventure, DelegateType::Query) => {
            let output = variant.output.as_ref().expect("output");
            (
                quote! {AdventureQueryData<#data, #output>},
                quote! {Option<&AdventureQuery<#data, #output>>},
            )
        }
    };

    let kind_name = Ident::new(&format!("{}Kind", enum_name), enum_name.span());

    quote! {
        #(#docs)*
        #[derive(Debug, Clone)]
        pub struct #struct_name<'a>(pub &'a #data);

        impl<'a> #trait_value for #struct_name<'a> {
            fn data(&self) -> &#data {
                &self.0
            }

            fn kind(&self) -> #kind_name {
                #kind_name::#name
            }

            fn extract(delegate: &#enum_name) -> #return_value {
                match delegate {
                    #enum_name::#name(d) => Some(d),
                    _ => None,
                }
            }
        }
    }
}

fn generic_argument(input: &AngleBracketedGenericArguments, index: usize) -> syn::Result<&Path> {
    let arg =
        input.args.iter().nth(index).ok_or_else(|| error("Missing expected generic parameter"))?;
    let path = match arg {
        GenericArgument::Type(Type::Path(p)) => p,
        _ => return Err(error("Expected GenericArgument::Type")),
    };
    Ok(&path.path)
}

fn error(message: &str) -> syn::Error {
    syn::Error::new(Span::call_site(), message)
}
