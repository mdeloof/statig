use std::collections::HashSet;

use syn::parse_quote;
use syn::visit::{self, Visit};
use syn::{GenericArgument, Generics, Type};

pub fn generic_candidates_from_type(ty: &Type) -> HashSet<GenericArgument> {
    let mut visitor = GenericCandidatesFromType::default();
    visitor.visit_type(ty);
    visitor.0
}

pub fn generic_candidates_from_generics(generics: &Generics) -> HashSet<GenericArgument> {
    let mut visitor = GenericCandidatesFromType::default();
    visitor.visit_generics(generics);
    visitor.0
}

#[derive(Default, Debug)]
struct GenericCandidatesFromType(HashSet<GenericArgument>);

impl<'ast> Visit<'ast> for GenericCandidatesFromType {
    fn visit_ident(&mut self, ident: &'ast proc_macro2::Ident) {
        self.0.insert(GenericArgument::Type(parse_quote!(#ident)));
        visit::visit_ident(self, ident);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast syn::Lifetime) {
        self.0.insert(GenericArgument::Lifetime(lifetime.clone()));
    }

    fn visit_type_param_bound(&mut self, _: &'ast syn::TypeParamBound) {}

    fn visit_const_param(&mut self, param: &'ast syn::ConstParam) {
        let ident = &param.ident;
        self.0.insert(GenericArgument::Type(parse_quote!(#ident)));
    }
}

#[test]
fn visit_type() {
    let ty: Type = parse_quote!(Vec<[&'a T; SIZE]>);
    let mut visitor = GenericCandidatesFromType::default();
    visitor.visit_type(&ty);
}

#[test]
fn visit_generics() {
    use syn::ItemImpl;

    let item_impl: ItemImpl = parse_quote!(
        impl<'a, T, const VALUE: usize> Foo<'a, T, VALUE> {}
    );
    let mut visitor = GenericCandidatesFromType::default();
    visitor.visit_generics(&item_impl.generics);
}
