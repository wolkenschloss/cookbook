use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, token, Block, Ident, ItemType, Stmt, Token};

/// Mit dem Spec Macro werden Testfälle beschrieben und ausführbare
/// Tests generiert.
///
///
/// Beim Testen von Software werden häufig unterschiedliche
/// Eingabedaten mit den selben Handlungen ausgeführt. Dieses Konzept
/// ist in Rust für die Tests nicht vorhanden. Es muss im Einzelfall
/// ausprogrammiert werden.
///
/// Mit dem Spec Macro werden die für einen Test unterschiedlichen
/// Eingabedaten von den auszuführenden Schritten getrennt
/// formuliert. Das Macro fügt dann zur Übersetzungszeit beides
/// in eigene Testmethoden zusammen.
///
/// # Offene Aufgaben
/// - [] Attribute `#[should_panic]` und `#[ignore]` an Testfällen
/// - [] Rückgabewert der generierten Testfunktion optional Result<>
///
/// # Syntax
///
/// Die Syntax für Spezifikationen, die vom Spec Macro akzeptiert
/// werden ist:
///
/// ```bnf
/// specification : ident '{' <case>+ <body> '}'
/// case : 'case' ident '{' <body> '}'
/// body : stmt*
/// ```
///
/// Ident für case muss eindeutig innerhalb der Spezifikation sein.
///
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
/// Das Beispiel erzeugt folgende Testfunktionen:
///
/// ```
/// mod example {
///   #[test]
///   fn test_1() {
///     let a = 1;
///     let want = 1;
///
///     let got = a * a;
///     assert_eq!(want, got);
///   }
///
///   // ...
/// }
///
#[proc_macro]
pub fn spec(input: TokenStream) -> TokenStream {
    let spec = parse_macro_input!(input as Spec);
    let spec_name = &spec.ident;
    let body = spec.body.stmts;
    let opt_ret_type = spec.body.output;

    let tests = spec.body.cases.into_iter().map(|c| {
        let ident = c.case_id;
        let prelude = c.stmts;

        match opt_ret_type {
            Some(ref ret_type) => {
                let ty = ret_type.ty.clone();
                quote! {
                    #[test]
                    fn #ident() -> #ty {
                        #(#prelude)*
                        #(#body)*
                    }
                }
            }
            None => {
                quote! {
                    #[test]
                    fn #ident() {
                        #(#prelude)*
                        #(#body)*
                    }
                }
            }
        }
    });

    TokenStream::from(quote! {
        #[cfg(test)]
        mod #spec_name {
            use super::*;

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
        Ok(Spec { ident, body })
    }
}

struct SpecBody {
    stmts: Vec<Stmt>,
    cases: Vec<Case>,
    output: Option<ItemType>,
}

impl Parse for SpecBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cases = Vec::new();

        let mut lookahead = input.lookahead1();
        let output = if lookahead.peek(Token![type]) {
            let o = input.call(syn::ItemType::parse).ok();
            // if let Some(ref p) = o {
            //     let p2 = p;
            //     let text = quote!{ #p2 };
            //     eprintln!("Got output type {}", text);
            // }
            lookahead = input.lookahead1();
            o
        } else {
            None
        };

        while lookahead.peek(kw::case) {
            let _case = input.parse::<kw::case>()?;
            let case_id: Ident = input.parse()?;

            let content;
            let _brace_token: token::Brace = braced!(content in input);
            let stmts = content.call(Block::parse_within)?;
            cases.push(Case { case_id, stmts });

            lookahead = input.lookahead1();
        }

        let stmts = Block::parse_within(input)?;

        Ok(SpecBody {
            cases,
            stmts,
            output,
        })
    }
}

mod kw {
    syn::custom_keyword!(case);
}

struct Case {
    case_id: Ident,
    stmts: Vec<Stmt>,
}
