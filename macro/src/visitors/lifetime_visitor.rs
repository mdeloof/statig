use proc_macro2::Span;
use syn::visit_mut::VisitMut;
use syn::Lifetime;
use syn::Type;

// Visit all lifetimes (both generic params and references) and rename them when they
// are anonymou. For instance `T<'_>` becomes `T<'sub>` and `&T` becomes `&'sub T`.
pub struct LifetimeVisitor {
    lifetime: Lifetime,
}

impl LifetimeVisitor {
    pub fn new(name: &str) -> Self {
        Self {
            lifetime: Lifetime::new(name, Span::call_site()),
        }
    }

    pub fn rename_type(&mut self, ty: &mut Type) {
        self.visit_type_mut(ty);
    }
}

impl VisitMut for LifetimeVisitor {
    fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
        if lifetime.ident != "'_" {
            *lifetime = self.lifetime.clone();
        }
    }

    fn visit_type_reference_mut(&mut self, reference: &mut syn::TypeReference) {
        match &mut reference.lifetime {
            Some(lifetime) if lifetime.ident == "'_" => *lifetime = self.lifetime.clone(),
            None => reference.lifetime = Some(self.lifetime.clone()),
            _ => (),
        }
    }
}

#[test]
fn lifetime_visitor() {
    use syn::parse_quote;

    let mut ty: Type = parse_quote!((Context<'_>, &T, &'static T));

    let mut lifetime = LifetimeVisitor::new("'sub");

    lifetime.rename_type(&mut ty);

    let expected = parse_quote!((Context<'sub>, &'sub T, &'static T));

    assert_eq!(ty, expected);
}
