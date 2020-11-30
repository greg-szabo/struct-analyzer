use quote::__private::TokenTree;
use std::ops::Deref;
use syn::{Attribute, GenericArgument, Path, PathArguments, ReturnType, Type};

pub fn get_idents_from_paths(path: &Path) -> Vec<String> {
    let mut ident: String = "".to_string();
    for path_segment in &path.segments {
        ident.push_str(format!("::{}", path_segment.ident).as_str());
    }
    ident = ident.strip_prefix("::").unwrap().to_string();
    path.segments
        .iter()
        .map(|path_segment| {
            let mut results = match &path_segment.arguments {
                PathArguments::None => Vec::<String>::new(),
                PathArguments::AngleBracketed(ab) => ab
                    .args
                    .iter()
                    .map(|ga| match ga {
                        GenericArgument::Type(t) => get_idents_from_types(t),
                        GenericArgument::Binding(b) => get_idents_from_types(&b.ty),
                        _ => Vec::new(),
                    })
                    .flatten()
                    .collect::<Vec<String>>(),
                PathArguments::Parenthesized(pga) => match &pga.output {
                    ReturnType::Default => Vec::new(),
                    ReturnType::Type(_, b) => get_idents_from_types(b.deref()),
                },
            };
            // Crude filter to remove simple types
            if ![
                "i64",
                "u64",
                "i32",
                "u32",
                "i16",
                "u16",
                "i8",
                "u8",
                "Option",
                "Vec",
                "bool",
                "Box",
                "String",
                "std::time::Duration",
                // Todo: the below types can be eliminated by parsing the imports.
                "PathBuf",  // std::path::PathBuf
                "BTreeMap", // std::collections::BTreeMap
            ]
            .contains(&ident.as_str())
            {
                results.push(ident.clone());
            }
            results
        })
        .flatten()
        .collect::<Vec<String>>()
}

pub fn get_idents_from_types(ty: &Type) -> Vec<String> {
    match &ty {
        Type::Path(p) => get_idents_from_paths(&p.path),
        Type::Array(a) => get_idents_from_types(a.elem.deref()),
        Type::BareFn(bare_fn) => match &bare_fn.output {
            ReturnType::Default => Vec::new(),
            ReturnType::Type(_, b) => get_idents_from_types(b.deref()),
        },
        Type::Group(group) => get_idents_from_types(group.elem.deref()),
        Type::Paren(p) => get_idents_from_types(p.elem.deref()),
        Type::Ptr(p) => get_idents_from_types(p.elem.deref()),
        Type::Reference(r) => get_idents_from_types(r.elem.deref()),
        Type::Slice(s) => get_idents_from_types(s.elem.deref()),
        Type::Tuple(t) => t
            .elems
            .iter()
            .map(|t| get_idents_from_types(t))
            .flatten()
            .collect(),
        _ => Vec::new(),
    }
}

pub fn is_ident_present(attributes: &[Attribute], ident: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| get_idents_from_paths(&attribute.path).contains(&ident.to_string()))
}

pub fn is_ident_with_token_present(attributes: &[Attribute], ident: &str, token: &str) -> bool {
    attributes.iter().any(|attribute| {
        if get_idents_from_paths(&attribute.path).contains(&ident.to_string()) {
            attribute.tokens.clone().into_iter().any(|tt| match tt {
                TokenTree::Group(group) => group.stream().into_iter().any(|tt| match tt {
                    TokenTree::Ident(ident) => ident == token,
                    _ => false,
                }),
                _ => false,
            })
        } else {
            false
        }
    })
}

pub fn camelcase_to_snakecase(s: &str) -> String {
    // Exceptions...
    if s == "AppHash" {
        return "hash".to_string();
    }
    if s == "Type" {
        return "msg_type".to_string();
    }

    // Implement CamelCase to snake_case conversion
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_uppercase() {
                if i != 0 {
                    format!("_{}", c.to_lowercase())
                } else {
                    c.to_lowercase().to_string()
                }
            } else {
                c.to_string()
            }
        })
        .collect()
}
