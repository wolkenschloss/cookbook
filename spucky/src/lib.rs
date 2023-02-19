use proc_macro::{TokenStream};
use quote::{quote};
use syn::parse::{Parse, ParseStream};
use syn::{Block, braced, Ident, parse_macro_input, Stmt, token};


/// Generiert Testfunktionen anhand von Testfällen
///
/// # Examples
///
/// ```
/// use spucky::spec;
///
///   spec! {
///     example {
///         case test_1 {
///             let a = 1;
///             let want = 1;
///         }
///         case test_4 {
///             let a = 4;
///             let want = 16;
///         }
///         case test_5 {
///             let a = 5;
///             let want = 24;
///         }
///
///         let got = a * a;
///         assert_eq!(want, got);
///     }
///   }
/// ```
///
#[proc_macro]
pub fn spec(input: TokenStream) -> TokenStream {

    let spec = parse_macro_input!(input as Spec);
    let spec_name = &spec.ident;
    let body = spec.body.stmts;

    let tests = spec.body.cases.into_iter().map(|c| {
        let ident = c.case_id;
        let prelude = c.stmts;

        quote! {
            #[test]
            fn #ident() {
                #(#prelude)*
                #(#body)*
            }
        }
    });

    TokenStream::from(quote!{
        #[cfg(test)]
        mod #spec_name {
            #(#tests)*
        }
    })
}

struct Spec {
    ident: Ident,
    body: SpecBody,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        let ident: Ident = input.parse()?;
        let _brace_token: token::Brace = braced!(content in input);

        let body = content.call(SpecBody::parse)?;
        Ok(Spec{ident, body})
    }
}

struct SpecBody {
    stmts: Vec<Stmt>,
    cases: Vec<Case>,
}

impl Parse for SpecBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cases = Vec::new();

        let mut lookahead = input.lookahead1();
        while lookahead.peek(kw::case) {
            let _case = input.parse::<kw::case>()?;
            let case_id: Ident = input.parse()?;

            let content;
            let _brace_token: token::Brace = braced!(content in input);
            let stmts = content.call(Block::parse_within)?;
            cases.push(Case {
                case_id,
                stmts
            });

            lookahead = input.lookahead1();
        }

        let stmts =  Block::parse_within(input)?;

        Ok(SpecBody{
            cases,
            stmts
        })
    }
}

mod kw {
    syn::custom_keyword!(case);
}

struct Case {
    case_id: Ident,
    stmts: Vec<Stmt>
}

