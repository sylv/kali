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

struct Field {
    name: Ident,
    variant_name: Ident,
    kind: syn::Type,
    is_pk: bool,
}

fn parse_fields(entity: &syn::ItemStruct) -> Vec<Field> {
    entity
        .fields
        .iter()
        .filter_map(|f| {
            let ident = f.ident.as_ref()?;
            let name = ident.clone();
            let variant_name = Ident::new(&to_upper_camel_case(&name.to_string()), name.span());
            let kind = f.ty.clone();

            // Check if field has a #[primary_key] attribute
            let is_pk = f
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("primary_key"));

            Some(Field {
                name,
                variant_name,
                kind,
                is_pk,
            })
        })
        .collect()
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

    let vis = entity.vis.clone();
    let name = entity.ident.clone();

    let fields = parse_fields(&entity); // Parse fields while attributes are still present

    // Remove #[primary_key] attribute from the entity's fields before outputting the struct
    match &mut entity.fields {
        syn::Fields::Named(fields_named) => {
            for field in fields_named.named.iter_mut() {
                field
                    .attrs
                    .retain(|attr| !attr.path().is_ident("primary_key"));
            }
        }
        syn::Fields::Unnamed(_) => {}
        syn::Fields::Unit => {}
    }

    // pk is either with #[primary_key] attribute or named "id"
    let primary_key_field = fields
        .iter()
        .find(|f| f.is_pk)
        .or_else(|| fields.iter().find(|f| f.name == "id"));

    let Some(primary_key_field) = primary_key_field else {
        return Err(syn::Error::new(
            name.span(),
            "No primary key field found. Use #[primary_key] attribute or a field named 'id'.",
        ));
    };

    let col_enum_name = Ident::new(&format!("{}Column", name), name.span());

    // Extract field names and variant names from the fields
    let field_names = fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
    let col_enum_variants = fields
        .iter()
        .map(|f| f.variant_name.clone())
        .collect::<Vec<_>>();

    let col_constants = col_enum_variants.iter().map(|variant| {
        quote! {
            pub const #variant: #col_enum_name = #col_enum_name::#variant;
        }
    });

    // Generate column enum match arms for the Column implementation
    let snake_case_field_names = field_names
        .iter()
        .map(|field_name| field_name.to_string())
        .collect::<Vec<_>>();

    let column_match_arms = col_enum_variants
        .iter()
        .zip(snake_case_field_names.iter())
        .map(|(variant, field_name)| {
            quote! {
                #col_enum_name::#variant => #field_name
            }
        });

    // Generate COLUMNS array with all column variants
    let columns_array = quote! {
        #vis const COLUMNS: &'static [#col_enum_name] = &[
            #(#col_enum_name::#col_enum_variants),*
        ];
    };

    // Generate primary key constant if a primary key was found
    let pk_constant = {
        let pk_variant = &primary_key_field.variant_name;
        quote! {
            #vis const PRIMARY_KEY: #col_enum_name = #col_enum_name::#pk_variant;
        }
    };

    let pk_type = &primary_key_field.kind;

    Ok(quote! {
        #entity // Now #entity will be quoted without #[primary_key] on its fields

        #[allow(non_upper_case_globals)]
        impl #name {
            #vis const TABLE_NAME: &'static str = #table_name;
            #columns_array
            #pk_constant
            #(#col_constants)*

            #vis async fn fetch_one<'e, E>(
                executor: E,
                id: #pk_type,
            ) -> Result<Self, sqlx::Error>
            where
                E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                kali::builder::QueryBuilder::select_from(Self::TABLE_NAME)
                    .columns(&[#(#col_enum_name::#col_enum_variants),*])
                    .filter(Self::PRIMARY_KEY.eq(id))
                    .limit(1)
                    .fetch_one(executor)
                    .await
            }

            #vis async fn fetch_optional<'e, E>(
                executor: E,
                id: #pk_type,
            ) -> Result<Option<Self>, sqlx::Error>
            where
                E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                kali::builder::QueryBuilder::select_from(Self::TABLE_NAME)
                    .columns(&[#(#col_enum_name::#col_enum_variants),*])
                    .filter(Self::PRIMARY_KEY.eq(id))
                    .limit(1)
                    .fetch_optional(executor)
                    .await
            }

            #vis async fn fetch_all<'e, E>(
                executor: E,
            ) -> Result<Vec<Self>, sqlx::Error>
            where
                E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                kali::builder::QueryBuilder::select_from(Self::TABLE_NAME)
                    .columns(&[#(#col_enum_name::#col_enum_variants),*])
                    .fetch_all(executor)
                    .await
            }

            #vis async fn delete_one<'e, E>(
                executor: E,
                id: #pk_type,
            ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>
            where
                E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                kali::builder::QueryBuilder::delete_from(Self::TABLE_NAME)
                    .filter(Self::PRIMARY_KEY.eq(id))
                    .execute(executor)
                    .await
            }

            #vis fn query<'a>() -> kali::builder::QueryBuilder<'a, kali::builder::Select, #col_enum_name> {
                kali::builder::QueryBuilder::select_from(Self::TABLE_NAME)
                    .columns(&[#(#col_enum_name::#col_enum_variants),*])
            }
        }

        #vis enum #col_enum_name {
            #(#col_enum_variants),*
        }

        impl kali::column::Column for #col_enum_name {
            fn raw(&self) -> &str {
                match self {
                    #(#column_match_arms),*
                }
            }
        }
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
