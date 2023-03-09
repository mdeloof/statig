use std::collections::HashSet;

use syn::visit::{self, Visit};
use syn::{GenericParam, Generics, LifetimeDef, PatType};

/// Visitor to find all the generic parameters in a function signature.
#[derive(Debug)]
pub struct GenericParamVisitor<'a> {
    candidates: &'a Generics,
    found: HashSet<GenericParam>,
}

impl<'a> GenericParamVisitor<'a> {
    pub fn new(candidates: &'a Generics) -> Self {
        Self {
            candidates,
            found: HashSet::new(),
        }
    }

    pub fn search(&mut self, args: impl std::iter::IntoIterator<Item = &'a PatType>) {
        for arg in args {
            self.visit_pat_type(arg);
        }
    }

    pub fn finish(self) -> HashSet<GenericParam> {
        self.found
    }
}

impl<'ast> Visit<'ast> for GenericParamVisitor<'ast> {
    fn visit_ident(&mut self, ident: &'ast proc_macro2::Ident) {
        for param in self.candidates.params.iter() {
            match param {
                GenericParam::Type(ty) if &ty.ident == ident => {
                    self.found.insert(param.clone());
                }
                GenericParam::Const(constant) if &constant.ident == ident => {
                    self.found.insert(param.clone());
                }
                _ => continue,
            }
        }
        visit::visit_ident(self, ident);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast syn::Lifetime) {
        let lifetime = LifetimeDef::new(lifetime.clone());
        self.found.insert(GenericParam::Lifetime(lifetime));
    }
}

#[test]
fn visit_type() {
    use syn::{parse_quote, Type};

    let ty: Type = parse_quote!(Vec<[&'a T; SIZE]>);
    let generics = Generics::default();
    let mut visitor = GenericParamVisitor::new(&generics);
    visitor.visit_type(&ty);
}

#[test]
fn visit_generics() {
    use syn::{parse_quote, ItemImpl};

    let item_impl: ItemImpl = parse_quote!(
        impl<'a, T, const VALUE: usize> Foo<'a, T, VALUE> {}
    );
    let generics = Generics::default();
    let mut visitor = GenericParamVisitor::new(&generics);
    visitor.visit_generics(&item_impl.generics);
}
