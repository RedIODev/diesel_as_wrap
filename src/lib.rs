

///
/// This macro helps creating wrapper types for use of not natively supported types in structs.
/// It is still required to write the serialization and Deserialization logic for the wrapped types.
/// This can be done inside this macro using simplified function syntax.\
/// This macro will create a module containing 2 types one named As and one named AsOption.\
/// These types can be used with the deserialize_as/serialize_as macro annotations.\
/// The As type can be used for NonNullable sqlTypes and the AsOption for Nullable.\
/// The types of the targeted struct fields should be your type or Option<your type> respectively.\
/// The syntax for using this macro is the following:\
/// ```
/// wrap! {
///     target = $type_to_wrap$;
///     sql_type = $type_represented_as_sql$;
///     $module_visability$ mod $name_of_wrappers_module$;
///     fn to_sql<$parsing_type$>(self, out){
///         ...
///         let foo: &$parsing_type$ = ...;
///         foo.to_sql(out)
///     }
///     fn from_sql<$parsing_type$>(bytes){
///         let value = <$parsing_type$>::from_sql(bytes)?;
///         ...
///     }
/// }
/// ```
/// 
/// - type_to_wrap is the type you like to wrap (Needs to be fully qualified).
/// - type_represented_as_sql is the sql type your type should be represented as.
/// - name_of_wrappers_module is the module created by this macro containing the wrappers.
/// - parsing_type is the type that can already be parsed by diesel and is closest to your type. 
/// It is used to deserialise the raw bytes from diesel and is used to return the bytes when serializing.
/// 
/// # Example:
/// ```
/// wrap! {
///     target = uuid::Uuid;
///     sql_type = Binary;
///     pub mod uuid_wrap;
///     fn to_sql<[u8]>(self, out){
///         let bytes: &[u8] = self.0.as_bytes();
///         bytes.to_sql(out)
///     }
///     fn from_sql<Vec<u8>>(bytes){
///         let value = <Vec<u8>>::from_sql(bytes)?;
///         uuid::Uuid::from_slice(&value)
///             .map(As)
///             .map_err(|e| e.into())
///     }
/// }
/// 
/// 
/// #[derive(Debug, Queryable, Selectable, Insertable)]
/// #[diesel(table_name = crate::schema::foo)]
/// pub struct Foo {
///     #[diesel(deserialize_as = uuid_wrap::As)]
///     #[diesel(serialize_as = uuid_wrap::As)]
///     id: Uuid,
///     #[diesel(deserialize_as = uuid_wrap::AsOption)]
///     #[diesel(serialize_as = uuid_wrap::AsOption)]
///     opt_id: Option<Uuid>,
/// }
/// ```
/// 
#[macro_export]
macro_rules! wrap {
    (target = $target:ty; sql_type = $sql_type:ty; $visablity:vis mod $name:ident; fn to_sql<$to_intermediate:ty>($self:ident, $out:ident)$to:block fn from_sql<$from_intermediate:ty>($bytes:ident)$from:block) => {

        $visablity mod $name {

            use std::option::Option;
            use diesel::sql_types::*;
            use diesel::{
                backend::Backend, deserialize::{
                    FromSql, Result as DResult
                }, serialize::{
                    ToSql, Result as SResult, Output, IsNull
                },
                FromSqlRow, AsExpression
            };

            ///Wrapper that can be used for #[diesel(serialize_as())] and #[diesel(deserialize_as())].
            #[derive(Debug, FromSqlRow, AsExpression)]
            #[diesel(sql_type = $sql_type)]
            pub struct As(pub $target);

            impl From<As> for $target {
                fn from(s: As) -> Self {
                    s.0
                }
            }

            impl From<$target> for As {
                fn from(s: $target) -> Self {
                    As(s)
                }
            }

            impl<B> FromSql<$sql_type, B> for As
            where
                B: Backend,
                $from_intermediate: FromSql<$sql_type, B>,
            {
                fn from_sql($bytes: B::RawValue<'_>) -> DResult<Self> $from
            }

            impl<B> ToSql<$sql_type, B> for As
            where
                B: Backend,
                $to_intermediate: ToSql<$sql_type, B>,
            {
                fn to_sql<'b>(&'b $self, $out: &mut Output<'b, '_, B>) -> SResult $to
            }

            ///Wrapper that can be used for #[diesel(serialize_as())] and #[diesel(deserialize_as())] for an optional database entry.
            #[derive(Debug, FromSqlRow, AsExpression)]
            #[diesel(sql_type = $sql_type)]
            pub struct AsOption(pub Option<As>);

            impl From<AsOption> for Option<$target> {
                fn from(s: AsOption) -> Self {
                    s.0.map(|w| w.0)
                }
            }

            impl From<Option<$target>> for AsOption {
                fn from(s: Option<$target>) -> Self {
                    AsOption(s.map(|u| As(u)))
                }
            }

            impl<B> FromSql<Nullable<$sql_type>, B> for AsOption
            where
                B: Backend,
                As: FromSql<$sql_type, B>,
            {
                fn from_sql(bytes: B::RawValue<'_>) -> DResult<Self> {
                    Ok(AsOption(<Option<As>>::from_sql(bytes)?))
                }
            }

            impl<B> ToSql<$sql_type, B> for AsOption
            where
                B: Backend,
                As: ToSql<$sql_type, B>,
            {
                fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, B>) -> SResult {
                    if let Some(uuid) = &self.0 {
                        uuid.to_sql(out)
                    } else {Ok(IsNull::Yes)}
                }
            }
        }
    };


}
