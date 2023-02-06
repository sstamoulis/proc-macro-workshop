use itertools::Itertools;
use proc_macro2::Span;
use syn::{
    AngleBracketedGenericArguments, Field, GenericArgument, Ident, Meta, MetaList, NestedMeta,
    PathArguments, PathSegment, Type,
};

pub trait Optional {
    fn get_optional_type(&self) -> Option<&Type>;
}

pub trait Each {
    fn get_each(&self) -> Option<(Ident, &Type)>;
}

pub trait GetIdent {
    fn get_ident(&self) -> Ident;
}

impl Optional for Field {
    fn get_optional_type(&self) -> Option<&Type> {
        match &self.ty {
            Type::Path(t) => match t.path.segments.iter().exactly_one() {
                Ok(s) => s
                    .ident
                    .eq("Option")
                    .then(|| get_generic_argument_type(&self.ty)),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

impl Each for Field {
    fn get_each(&self) -> Option<(Ident, &Type)> {
        match self.attrs.iter().at_most_one() {
            Ok(Some(attr)) if attr.path.is_ident("builder") => match attr.parse_meta() {
                Ok(Meta::List(MetaList { nested, .. })) => match nested.iter().exactly_one() {
                    Ok(NestedMeta::Meta(Meta::NameValue(nv))) if nv.path.is_ident("each") => {
                        let ident = match &nv.lit {
                            syn::Lit::Str(lit) => Ident::new(&lit.value(), Span::call_site()),
                            _ => unimplemented!(),
                        };
                        let ty = get_generic_argument_type(&self.ty);
                        Some((ident, ty))
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            },
            Ok(None) => None,
            _ => unimplemented!(),
        }
    }
}

impl GetIdent for Field {
    fn get_ident(&self) -> Ident {
        self.ident
            .clone()
            .unwrap_or_else(|| unimplemented!("Field must have ident"))
    }
}

fn get_generic_argument_type(ty: &Type) -> &Type {
    match ty {
        Type::Path(t) => match t.path.segments.iter().exactly_one() {
            Ok(PathSegment {
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
                ..
            }) => match args.iter().exactly_one() {
                Ok(GenericArgument::Type(t)) => t,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
