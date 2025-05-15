use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

#[proc_macro_attribute]
pub fn entity(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match generate(args.into(), input.into()) {
        Ok(output) => output.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[derive(Clone)]
enum Relation {
    ForeignKey {
        entity: Ident,
        foreign_key_field: Ident,
        references_field: Option<Ident>, // defaults to primary key
    },
    ReferencedBy {
        entity: Ident,
        relation_field: Ident,
        is_collection: bool,
    },
}

#[derive(Clone)]
struct ParsedField {
    field_name: Ident,
    iden_name: Ident,
    is_pk: bool,
    relation: Option<Relation>,
    raw: syn::Field,
}

// parse Collection<T> or Reference<T> to T
fn parse_entity_from_type(entity: &syn::Type) -> Result<(bool, Ident), syn::Error> {
    if let syn::Type::Path(type_path) = entity {
        if let Some(segment) = type_path.path.segments.last() {
            let is_collection = segment.ident == "Collection";
            if is_collection || segment.ident == "Reference" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                        if let syn::Type::Path(type_path) = ty {
                            return Ok((
                                is_collection,
                                type_path.path.segments.last().unwrap().ident.clone(),
                            ));
                        }
                    }
                }
            }
        }
    }
    Err(syn::Error::new_spanned(
        entity,
        "expected Collection<T> or Reference<T>",
    ))
}

fn parse_fields(entity: syn::ItemStruct) -> Result<Vec<ParsedField>, syn::Error> {
    entity
        .fields
        .into_iter()
        .map(|mut f| {
            let ident = f
                .ident
                .as_ref()
                .ok_or_else(|| syn::Error::new_spanned(&f, "expected named field"))?;

            let field_name = ident.clone();
            let iden_name = Ident::new(
                &to_upper_camel_case(&field_name.to_string()),
                field_name.span(),
            );

            let mut is_pk = false;
            let mut relation_attr = None;

            f.attrs.retain(|attr| {
                if attr.path().is_ident("primary_key") {
                    is_pk = true;
                    false
                } else if attr.path().is_ident("relation") {
                    relation_attr = Some(attr.clone());
                    false
                } else {
                    true
                }
            });

            let relation = if let Some(relation_attr) = relation_attr {
                let (is_collection, entity) = parse_entity_from_type(&f.ty)?;
                let is_owning = relation_attr.path().is_ident("foreign_key");
                if is_owning {
                    return Err(syn::Error::new_spanned(
                        &relation_attr,
                        "expected `#[relation(referenced_by = ...)]` for Collection<T>",
                    ));
                }

                let mut referenced_by = None;
                let mut foreign_key = None;
                let mut references = None;
                relation_attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("referenced_by") {
                        let value = meta.value()?;
                        referenced_by = Some(value.parse()?);
                        Ok(())
                    } else if meta.path.is_ident("foreign_key") {
                        let value = meta.value()?;
                        foreign_key = Some(value.parse()?);
                        Ok(())
                    } else if meta.path.is_ident("references") {
                        let value = meta.value()?;
                        references = Some(value.parse()?);
                        Ok(())
                    } else {
                        return Err(syn::Error::new_spanned(
                            &meta.path,
                            "expected `referenced_by` or `foreign_key` attribute",
                        ));
                    }
                })?;

                match (referenced_by, foreign_key, references) {
                    (None, Some(fk), None) => {
                        Some(Relation::ForeignKey {
                            entity,
                            foreign_key_field: fk,
                            references_field: None,
                        })
                    }
                    (None, Some(fk), Some(refs)) => {
                        Some(Relation::ForeignKey {
                            entity,
                            foreign_key_field: fk,
                            references_field: Some(refs),
                        })
                    },
                    (Some(refs), None, None) => {
                        Some(Relation::ReferencedBy {
                            entity,
                            relation_field: refs,
                            is_collection,
                        })
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &relation_attr,
                            "expected either `#[relation(referenced_by = ...)]` or `#[relation(foreign_key = ...)]`",
                        ));
                    }
                }
            } else {
                None
            };

            Ok(ParsedField {
                field_name,
                iden_name,
                is_pk,
                relation,
                raw: f,
            })
        })
        .collect()
}

fn generate_entity_column_enum(
    vis: &syn::Visibility,
    entity_name: &syn::Ident,
    parsed_fields: &[ParsedField],
) -> syn::Result<(Ident, TokenStream)> {
    let col_enum_ident = Ident::new(&format!("{}Column", entity_name), entity_name.span());

    let col_enum_variants = parsed_fields
        .iter()
        .map(|f| f.iden_name.clone())
        .collect::<Vec<_>>();

    // Create a mapping of enum variant to field name for the match expression
    let field_name_mappings = parsed_fields
        .iter()
        .map(|f| {
            let variant = &f.iden_name;
            let field_name = &f.field_name;
            quote! { #col_enum_ident::#variant => stringify!(#field_name) }
        })
        .collect::<Vec<_>>();

    let col_enum = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #vis enum #col_enum_ident {
            #(#col_enum_variants),*
        }

        impl kali::column::Column for #col_enum_ident {
            fn to_col_name(&self) -> &str {
                match self {
                    #(#field_name_mappings),*
                }
            }
        }
    };

    Ok((col_enum_ident, col_enum))
}

fn generate_entity_constants(
    vis: &syn::Visibility,
    entity_enum_ident: &Ident,
    parsed_fields: &[ParsedField],
    primary_key: &ParsedField,
) -> syn::Result<TokenStream> {
    let col_enum_variants = parsed_fields
        .iter()
        .map(|f| f.iden_name.clone())
        .collect::<Vec<_>>();

    let col_constants = parsed_fields
        .iter()
        .map(|f| {
            let iden_name = &f.iden_name;
            quote! {
                #[allow(non_upper_case_globals)]
                pub const #iden_name: #entity_enum_ident = #entity_enum_ident::#iden_name;
            }
        })
        .collect::<Vec<_>>();

    let primary_key_iden_name = &primary_key.iden_name;

    Ok(quote! {
        #vis const COLUMNS: &'static [#entity_enum_ident] = &[#(#entity_enum_ident::#col_enum_variants),*];
        #vis const PRIMARY_KEY: #entity_enum_ident = #entity_enum_ident::#primary_key_iden_name;
        #(#col_constants)*
    })
}

fn generate_relation_functions(
    entity_name: &Ident,
    col_enum_name: &Ident,
    vis: &syn::Visibility,
    parsed_relations: &[ParsedField],
) -> syn::Result<TokenStream> {
    let relation_functions = parsed_relations
        .iter()
        .map(|f| {
            let relation = f.relation.as_ref().unwrap();
            match relation {
                Relation::ForeignKey {
                    entity: inversed_entity,
                    foreign_key_field,
                    references_field,
                } => {
                    let field_name = &f.field_name;
                    let references_field = references_field
                        .as_ref();

                    let inversed_primary_key_getter = match references_field {
                        Some(refs) => {
                            quote! { entity.#refs }
                        },
                        None => {
                            quote! { entity.__primary_key_value() }
                        },
                    };

                    let references_field_iden_ident = if let Some(refs) = references_field {
                        Ident::new(
                            &to_upper_camel_case(&refs.to_string()),
                            refs.span(),
                        )
                    } else {
                        Ident::new(
                            "PRIMARY_KEY",
                            foreign_key_field.span(),
                        )
                    };

                    let foreign_key_iden_ident = Ident::new(
                        &to_upper_camel_case(&foreign_key_field.to_string()),
                        foreign_key_field.span(),
                    );

                    let inversed_filter_name = Ident::new(
                        &format!("__{}_inversed_filter", field_name),
                        field_name.span(),
                    );


                    quote! {
                        #vis fn #field_name(&self) -> kali::reference::Reference<#inversed_entity> {
                            kali::reference::Reference::new(#inversed_entity::#references_field_iden_ident.eq(self.#foreign_key_field))
                        }

                        // this is really awkward, but its necessary for the inversed side to know
                        // how to filter the relation. when the macro runs, we can't inspect the owning side
                        // to figure it out, and other workarounds aren't as clean.
                        #[doc(hidden)]
                        #vis fn #inversed_filter_name<'a>(entity: &#inversed_entity) -> kali::builder::expr::Expr<'a, #col_enum_name> {
                            #entity_name::#foreign_key_iden_ident.eq(#inversed_primary_key_getter)
                        }
                    }
                }
                Relation::ReferencedBy {
                    entity: owning_entity,
                    relation_field,
                    is_collection,
                } => {
                    // we use the inversed_filter to filter the relation appropriately
                    let field_name = &f.field_name;
                    let inversed_filter_name = Ident::new(
                        &format!("__{}_inversed_filter", relation_field),
                        relation_field.span(),
                    );

                    let return_kind = if *is_collection {
                        quote! { kali::collection::Collection<#owning_entity> }
                    } else {
                        quote! { kali::reference::Reference<#owning_entity> }
                    };

                    let struct_kind = if *is_collection {
                        quote! { kali::collection::Collection }
                    } else {
                        quote! { kali::reference::Reference }
                    };

                    quote! {
                        #vis fn #field_name(&self) -> #return_kind {
                            #struct_kind::new(#owning_entity::#inversed_filter_name(self))
                        }
                    }

                } 
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        #(#relation_functions)*
    })
}

fn generate(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let mut entity: syn::ItemStruct = syn::parse2(input)?; // Make entity mutable

    // table name is either #[entity("table_name")] or snake_case of the struct name
    let table_name = if args.is_empty() {
        let name = entity.ident.to_string();
        let snake_case_name = to_snake_case(&name);
        quote! { #snake_case_name }
    } else {
        let table_name: syn::LitStr = syn::parse2(args)?;
        quote! { #table_name }
    };

    let entity_vis = entity.vis.clone();
    let entity_name = entity.ident.clone();

    let parsed_fields = parse_fields(entity.clone())?;
    let (parsed_fields, relation_fields): (Vec<_>, Vec<_>) = parsed_fields
        .into_iter()
        .partition(|f| f.relation.is_none());

    match entity.fields {
        syn::Fields::Named(ref mut fields) => {
            fields.named = parsed_fields.clone().into_iter().map(|f| f.raw).collect();
        }
        syn::Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(&entity, "expected named fields"));
        }
        syn::Fields::Unit => {
            return Err(syn::Error::new_spanned(&entity, "expected named fields"));
        }
    }

    // pk is either with #[primary_key] attribute or named "id"
    let primary_key = parsed_fields
        .iter()
        .find(|f| f.is_pk)
        .or_else(|| parsed_fields.iter().find(|f| f.field_name == "id"));

    let Some(primary_key) = primary_key else {
        return Err(syn::Error::new_spanned(
            &entity,
            "missing primary key field with #[primary_key] attribute or named 'id'",
        ));
    };

    let primary_key_name = &primary_key.field_name;
    let primary_key_type = &primary_key.raw.ty;
    let (col_enum_name, col_enum) =
        generate_entity_column_enum(&entity_vis, &entity_name, &parsed_fields)?;
    let entity_constants =
        generate_entity_constants(&entity_vis, &col_enum_name, &parsed_fields, primary_key)?;

    let relation_functions = generate_relation_functions(
        &entity_name,
        &col_enum_name,
        &entity_vis,
        &relation_fields,
    )?;

    Ok(quote! {
        #entity

        #[allow(non_upper_case_globals)]
        impl #entity_name {
            #entity_vis const TABLE_NAME: &'static str = #table_name;
            #entity_constants

            #relation_functions

            #entity_vis async fn fetch_one<'e, E>(
                executor: E,
                id: #primary_key_type,
            ) -> Result<Self, sqlx::Error>
            where
                E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                kali::builder::QueryBuilder::select_from(Self::TABLE_NAME)
                    .columns(Self::COLUMNS)
                    .filter(Self::PRIMARY_KEY.eq(id))
                    .limit(1)
                    .fetch_one(executor)
                    .await
            }

            #entity_vis async fn fetch_optional<'e, E>(
                executor: E,
                id: #primary_key_type,
            ) -> Result<Option<Self>, sqlx::Error>
            where
                E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                kali::builder::QueryBuilder::select_from(Self::TABLE_NAME)
                    .columns(Self::COLUMNS)
                    .filter(Self::PRIMARY_KEY.eq(id))
                    .limit(1)
                    .fetch_optional(executor)
                    .await
            }

            #entity_vis async fn fetch_all<'e, E>(
                executor: E,
            ) -> Result<Vec<Self>, sqlx::Error>
            where
                E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                kali::builder::QueryBuilder::select_from(Self::TABLE_NAME)
                    .columns(Self::COLUMNS)
                    .fetch_all(executor)
                    .await
            }

            #entity_vis async fn delete_one<'e, E>(
                executor: E,
                id: #primary_key_type,
            ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>
            where
                E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                kali::builder::QueryBuilder::delete_from(Self::TABLE_NAME)
                    .filter(Self::PRIMARY_KEY.eq(id))
                    .execute(executor)
                    .await
            }

            #[doc(hidden)]
            #entity_vis fn __primary_key_value(&self) -> #primary_key_type {
                self.#primary_key_name
            }
        }

        impl kali::entity::Entity for #entity_name {
            type C = #col_enum_name;

            fn table_name() -> &'static str {
                Self::TABLE_NAME
            }

            fn columns() -> &'static [#col_enum_name] {
                Self::COLUMNS
            }

            fn primary_key() -> &'static #col_enum_name {
                &Self::PRIMARY_KEY
            }
        }

        #col_enum
    })
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;

    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 && !prev_was_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_was_upper = true;
        } else {
            result.push(c);
            prev_was_upper = false;
        }
    }

    result
}

fn to_upper_camel_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}
