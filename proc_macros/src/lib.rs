extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Error};
use syn::parse::{Parse, ParseStream, Result};
use syn::Token;
use syn::punctuated::Punctuated;
use proc_macro2::{Span, Ident};

#[proc_macro_attribute]
pub fn derive_io(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let item = match syn::parse::<syn::Item>(input) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };
    let data = match item {
        syn::Item::Struct(s) => s,
        _ => return Error::new(Span::call_site(), "expect struct!").to_compile_error().into(),
    };
    let name = &data.ident;

    let defaults = match args.defaults() {
        Ok(d) => d,
        Err(e) => return e.to_compile_error().into(),
    }
        .iter()
        .map(|t| {
            t.expand(&name)
        })
        .fold(quote! {}, |acc, q| {
            quote! {
                #acc
                #q
            }
        });
    (quote! {
        #data
        #defaults
        impl IOAccess for #name {}
    }).into()
}


mod args_kw {
    syn::custom_keyword!(U8);
    syn::custom_keyword!(U16);
    syn::custom_keyword!(U32);
    syn::custom_keyword!(U64);
    syn::custom_keyword!(Bytes);
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum AccessTrait {
    U8,
    U16,
    U32,
    U64,
    Bytes,
}

impl AccessTrait {
    fn trait_name(&self) -> Ident {
        match self {
            AccessTrait::U8 => Ident::new("U8Access", Span::call_site()),
            AccessTrait::U16 => Ident::new("U16Access", Span::call_site()),
            AccessTrait::U32 => Ident::new("U32Access", Span::call_site()),
            AccessTrait::U64 => Ident::new("U64Access", Span::call_site()),
            AccessTrait::Bytes => Ident::new("BytesAccess", Span::call_site()),
        }
    }

    fn msg(&self, name: &Ident, method: &str) -> String {
        format!("{}::{} for {} not implement!", self.trait_name().to_string(), method, name.to_string())
    }

    fn expand(&self, name: &Ident) -> proc_macro2::TokenStream {
        let trait_name = self.trait_name();
        let write_msg = self.msg(name, "write");
        let read_msg = self.msg(name, "read");
        let content = match self {
            AccessTrait::U8 => quote! {
                    fn write(&self, addr: u64, _: u8) -> region::Result<()> {
                        Err(region::Error::AccessErr(addr, #write_msg.to_string()))
                    }

                    fn read(&self, addr: u64) -> region::Result<u8> {
                        Err(region::Error::AccessErr(addr, #read_msg.to_string()))
                    }
            },
            AccessTrait::U16 => quote! {
                fn write(&self, addr: u64, _: u16) -> region::Result<()> {
                        Err(region::Error::AccessErr(addr, #write_msg.to_string()))
                }

                fn read(&self, addr: u64)-> region::Result<u16> {
                        Err(region::Error::AccessErr(addr, #read_msg.to_string()))
                }
            },
            AccessTrait::U32 => quote! {
                fn write(&self, addr: u64, _: u32) -> region::Result<()> {
                        Err(region::Error::AccessErr(addr, #write_msg.to_string()))
                }

                fn read(&self, addr: u64)-> region::Result<u32> {
                        Err(region::Error::AccessErr(addr, #read_msg.to_string()))
                }
            },
            AccessTrait::U64 => quote! {
                fn write(&self, addr: u64, _: u64) -> region::Result<()> {
                        Err(region::Error::AccessErr(addr, #write_msg.to_string()))
                }

                fn read(&self, addr: u64)-> region::Result<u64> {
                        Err(region::Error::AccessErr(addr, #read_msg.to_string()))
                }
            },
            AccessTrait::Bytes => quote! {
                fn write(&self, addr: u64, _: &[u8]) -> region::Result<()> {
                        Err(region::Error::AccessErr(addr, #write_msg.to_string()))
                }

                fn read(&self, addr: u64, _: &mut [u8]) -> region::Result<()>  {
                        Err(region::Error::AccessErr(addr, #read_msg.to_string()))
                }
            },
        };
        quote! {
            impl #trait_name for #name {
                #content
            }
        }
    }
}

impl Parse for AccessTrait {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(args_kw::U8) {
            input.parse::<args_kw::U8>()?;
            Ok(AccessTrait::U8)
        } else if lookahead.peek(args_kw::U16) {
            input.parse::<args_kw::U16>()?;
            Ok(AccessTrait::U16)
        } else if lookahead.peek(args_kw::U32) {
            input.parse::<args_kw::U32>()?;
            Ok(AccessTrait::U32)
        } else if lookahead.peek(args_kw::U64) {
            input.parse::<args_kw::U64>()?;
            Ok(AccessTrait::U64)
        } else if lookahead.peek(args_kw::Bytes) {
            input.parse::<args_kw::Bytes>()?;
            Ok(AccessTrait::Bytes)
        } else {
            Err(lookahead.error())
        }
    }
}

struct Args(Punctuated<AccessTrait, Token![,]>);

impl Args {
    fn defaults(&self) -> Result<Vec<AccessTrait>> {
        let all_traits = vec![AccessTrait::U8, AccessTrait::U16, AccessTrait::U32, AccessTrait::U64, AccessTrait::Bytes];
        if self.0.is_empty() {
            Err(Error::new(Span::call_site(), "At least one in [U8|U16|U32|U64|Bytes]!"))
        } else {
            Ok(all_traits.iter().filter(|&t| {
                !self.0.iter().any(|&a| a == *t)
            }).map(|t| { t.clone() }).collect::<Vec<_>>())
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Args(input.parse_terminated(AccessTrait::parse)?))
    }
}
