/*
 * Mostly vibe-coded attribute-like macro that allows us to retrieve a function's name and the names
 * of its parameters at compile time. Used for generating useful error messages across the FFI boundary.
 */

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use std::ffi::CString;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, Pat, PatType};

fn c_str_lit(s: &str) -> Literal {
    let cstring = CString::new(s).expect("identifier unexpectedly contained a NUL byte");
    Literal::c_string(&cstring)
}

#[proc_macro_attribute]
pub fn c_interface_reflect(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let ItemFn { attrs, vis, sig, block, .. } = input_fn;

    // const FN_NAME: *const c_char = c"the_function_name".as_ptr();
    let fn_name_lit = c_str_lit(&sig.ident.to_string());
    let fn_const = quote! {
        const FN_NAME: *const ::std::ffi::c_char = #fn_name_lit.as_ptr();
    };

    // const ARG_<NAME>: *const c_char = c"name".as_ptr(); for each param
    let mut arg_consts = Vec::new();
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(_) => continue, // skip self / &self / &mut self
            FnArg::Typed(PatType { pat, .. }) => match pat.as_ref() {
                Pat::Wild(_) => continue, // `_: T` has no name to emit
                Pat::Ident(pat_ident) => {
                    let arg_name = pat_ident.ident.to_string();
                    let const_name = format!("ARG_{}", arg_name.to_uppercase());
                    let const_ident = Ident::new(&const_name, pat_ident.ident.span());
                    let arg_lit = c_str_lit(&arg_name);
                    arg_consts.push(quote! {
                        const #const_ident: *const ::std::ffi::c_char = #arg_lit.as_ptr();
                    });
                }
                other => {
                    return syn::Error::new_spanned(
                        other,
                        "c_str_consts: arguments must be simple identifiers, not destructuring patterns",
                    )
                        .to_compile_error()
                        .into();
                }
            },
        }
    }

    let stmts = block.stmts;

    quote! {
        #(#attrs)*
        #vis #sig {
            #fn_const
            #(#arg_consts)*
            #(#stmts)*
        }
    }
        .into()
}

#[proc_macro]
pub fn prefix_error_type(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as Ident);
    quote! { crate::interface::error_handling::ErrorType::#name }.into()
}

#[proc_macro]
pub fn arg_name_const_identifier(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as Ident);
    let upper = format!("ARG_{}", name.to_string().to_uppercase());
    let new_ident = Ident::new(&upper, name.span());
    quote! { #new_ident }.into()
}
