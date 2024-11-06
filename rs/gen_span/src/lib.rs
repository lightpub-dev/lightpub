use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut};

#[proc_macro_attribute]
pub fn gen_span(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let item = parse_macro_input!(input as syn::Item);

    process_fn(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn process_fn(mut item: syn::Item) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut span_gen = SpanGen::new();
    span_gen.visit_item_mut(&mut item);

    Ok(proc_macro2::TokenStream::from(quote! {
        #item
    }))
}

struct SpanGen {}

impl SpanGen {
    fn new() -> Self {
        SpanGen {}
    }
}

impl VisitMut for SpanGen {
    fn visit_item_fn_mut(&mut self, i: &mut syn::ItemFn) {
        // let stmts = i.block.stmts.clone();
        let fn_name_ident = &i.sig.ident;
        let fn_name = quote! {
            stringify!(#fn_name_ident)
        };

        let block = i.block.clone();
        // let end_block = quote! {
        //     {
        //         __otlp_fn_span.end();
        //     }
        // };
        // let end_block_stmt = syn::parse2(end_block).expect("failed to parse end_block");

        // match block.stmts.len() {
        //     0 => {
        //         block.stmts.push(end_block_stmt);
        //     }
        //     _ => {
        //         let last_stmt = block.stmts.pop().unwrap();
        //         block.stmts.push(end_block_stmt);
        //         block.stmts.push(last_stmt);
        //     }
        // }

        let new_block = quote! {
            {
                use opentelemetry::global::tracer;
                use opentelemetry::trace::Tracer;
                use opentelemetry::trace::TraceContextExt;
                let __otlp_tracer = opentelemetry::global::tracer("");
                let __otlp_parent_cx = opentelemetry::Context::current();
                let __otlp_fn_span = __otlp_tracer.start_with_context(#fn_name, &__otlp_parent_cx);
                let __otlp_cx = opentelemetry::Context::current_with_span(__otlp_fn_span);
                #block
            }
        };

        i.block = syn::parse2(new_block).expect("failed to parse");

        self.visit_block_mut(&mut i.block);
    }

    fn visit_impl_item_fn_mut(&mut self, i: &mut syn::ImplItemFn) {
        // let stmts = i.block.stmts.clone();
        let fn_name_ident = &i.sig.ident;
        let fn_name = quote! {
            stringify!(#fn_name_ident)
        };

        let block = i.block.clone();
        // let end_block = quote! {
        //     {
        //         __otlp_fn_span.end();
        //     }
        // };
        // let end_block_stmt = syn::parse2(end_block).expect("failed to parse end_block");

        // match block.stmts.len() {
        //     0 => {
        //         block.stmts.push(end_block_stmt);
        //     }
        //     _ => {
        //         let last_stmt = block.stmts.pop().unwrap();
        //         block.stmts.push(end_block_stmt);
        //         block.stmts.push(last_stmt);
        //     }
        // }

        let new_block = quote! {
            {
                use opentelemetry::global::tracer;
                use opentelemetry::trace::Tracer;
                use opentelemetry::trace::TraceContextExt;
                let __otlp_tracer = opentelemetry::global::tracer("");
                let __otlp_parent_cx = opentelemetry::Context::current();
                let __otlp_fn_span = __otlp_tracer.start_with_context(#fn_name, &__otlp_parent_cx);
                let __otlp_cx = opentelemetry::Context::current_with_span(__otlp_fn_span);
                #block
            }
        };

        i.block = syn::parse2(new_block).expect("failed to parse");

        self.visit_block_mut(&mut i.block);
    }

    fn visit_expr_await_mut(&mut self, i: &mut syn::ExprAwait) {
        let base = i.base.clone();
        let new_base = quote! {
            opentelemetry::trace::FutureExt::with_context(
                #base,
                __otlp_cx.clone()
            )
        };

        i.base = syn::parse2(new_base).expect("failed to parse");
    }
}
