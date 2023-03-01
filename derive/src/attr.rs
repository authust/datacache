use std::ops::Deref;

use syn::{Attribute, Error, Field, MetaList, NestedMeta};

pub fn find_attribute(attributes: &[Attribute]) -> Option<&Attribute> {
    attributes
        .into_iter()
        .find(|attr| attr.path.is_ident("datacache"))
}

#[inline(always)]
fn get_meta_list(attr: &Attribute) -> Result<MetaList, Error> {
    match attr.parse_meta()? {
        syn::Meta::List(list) => Ok(list),
        other => Err(Error::new_spanned(other, "unsupported attribute")),
    }
}

pub fn filter_attributes<A: Deref<Target = Attribute>>(
    iter: impl Iterator<Item = A>,
) -> impl Iterator<Item = A> {
    iter.filter(|attr| !attr.path.is_ident("datacache"))
}

#[derive(Clone)]
pub struct FieldAttr {
    pub queryable: bool,
}

pub fn field_attr(field: &Field) -> Result<FieldAttr, Error> {
    let mut field_data = FieldAttr { queryable: false };
    let attr = match find_attribute(&field.attrs) {
        Some(attr) => attr,
        None => return Ok(field_data),
    };
    let meta = get_meta_list(attr)?;
    for nested in meta.nested {
        if let NestedMeta::Meta(meta) = nested {
            match meta {
                syn::Meta::Path(path) => match path.get_ident() {
                    Some(ident) => match ident.to_string().as_str() {
                        "queryable" => field_data.queryable = true,
                        other => {
                            return Err(Error::new_spanned(
                                ident,
                                format!("unsupported attribute ({other})"),
                            ))
                        }
                    },
                    None => return Err(Error::new_spanned(path, "unsupported attribute")),
                },
                _ => return Err(Error::new_spanned(meta, "unsupported attribute")),
            }
        }
    }
    Ok(field_data)
}
