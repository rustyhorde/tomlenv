// Copyright (c) 2018 tomlenv developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tomlenv` allows you to drive your environment configuration from TOML.
//! The `Environments` struct holds a reference from your environment hierarchy
//! to the configuraion associated with that particular enviornment i.e.
//!
//! * `prod_key` -> `prod_config`
//! * `stage_key` -> `stage_config`
//! * `dev_key` -> `dev_config`
//!
//! On the key side, you can use the `Environment` hierarchy defined by this
//! library (Prod -> Stage -> Test -> Dev -> Local), or you can define your own
//! custom hierarchy to use with the `Environments` struct.  If your define a
//! custom hierarchy you must implement the `Deserialize`, `Serialize`, `Ord`,
//! `PartialOrd`, and `TryFrom<String>` traits.  See more in the
//! [Custom Environment Hierarchy](#custom-environment-hierarchy) section below.
//!
//! # Usage
//! First, define a struct that represents your environment configuration.  For
//! items that appear in one environment, but not another, use `Option`.  See
//! the key field in the example below.
//!
//! Second, generate a `Reader` from your TOML.  Normally, the TOML would be
//! defined as a `Path`, and used with the `from_path` method.  You can also
//! supply a generic `Read` implementation to the `from_reader` method, as
//! below.
//!
//! Third, deserialize the TOML into your `Environments` struct.  At this point,
//! you can use the `current` method to access your environment config for the
//! environment specified by the environment variable `env`.
//!
//! ```
//! # use tomlenv::{Environment, Environments, Error, Result};
//! # use getset::Getters;
//! # use serde::{Deserialize, Serialize};
//! # use std::env;
//! # use std::io::Cursor;
//! #
//! # fn foo() -> Result<()> {
//! /// Define your environment specific configuration.
//! /// *NOTE*: This must implement `Deserialize` and `Serialize`
//! #[derive(Debug, Deserialize, Getters, Serialize)]
//! struct MyAppEnv {
//!   /// The display name of this environment.
//!   #[get]
//!   name: String,
//!   /// The secret key only used in the Prod environment.
//!   #[get]
//!   key: Option<String>,
//! }
//!
//! /// Grab your environment TOML.  This would usually be in a file and can
//! /// be read to a string such as below.
//! let toml = r#"[envs.prod]
//! name = "Production"
//! key = "abcd-123-efg-45"
//!
//! [envs.stage]
//! name = "Stage"
//!
//! [envs.test]
//! name = "Test"
//!
//! [envs.dev]
//! name = "Development"
//!
//! [envs.local]
//! name = "Local"
//! "#;
//!
//! // Deserialize the TOML config into your environment structs.  This example
//! // is using the `Enrivorment` hierarchy supplied by the library.
//! let mut cursor = Cursor::new(toml);
//! let envs: Environments<Environment, MyAppEnv> = Environments::from_reader(&mut cursor)?;
//!
//! // Check the `Production` environment.
//! env::set_var("env", "prod");
//! let mut current = envs.current()?;
//! assert_eq!(current.name(), "Production");
//! assert_eq!(current.key(), &Some("abcd-123-efg-45".to_string()));
//!
//! // Switch to the `Development` environment.
//! env::set_var("env", "dev");
//! current = envs.current()?;
//! assert_eq!(current.name(), "Development");
//! assert_eq!(current.key(), &None);
//! #   Ok(())
//! # }
//! ```
//!
//! # Custom Environment Hierarchy
//! If you wish to forego using the `Environment` hierarchy supplied by this
//! library, implement a custom hierarchy instead.  There are a few traits you
//! must implement in order to work with `Environments`.
//!
//! ## Required
//! * `Deserialize` and `Serialize`: These are required to translate
//! to/from TOML.
//! * `Ord` and `PartialOrd`:  These are required to maintain proper ordering for
//! your hierarchy and ensure serialized TOML is always in the same order.
//! * `TryFrom<String>`: This is used to translate the environment variable `env`
//! into your hierarchy type.
//!
//! ## Optional
//! * `Display`: Used to convert your hierarchy type to a formatted string.
//! Useful if you need to show what environment you are using.
//! * `TryFrom<&'a str>`: In this case, used by the custom deserializer.
//!
//! Below is an example of a custom hierarchy.  This example has a custom
//! serializer/deserializer, but that shouldn't be necessary in all cases.
//!
//! ```
//! # use getset::Getters;
//! # use serde::{Deserialize as De, Serialize as Ser};
//! # use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
//! # use std::convert::TryFrom;
//! # use std::env;
//! # use std::fmt;
//! # use std::io::Cursor;
//! # use tomlenv::{Environments, Error, Result};
//! #
//! #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
//! pub enum MyHierarchy {
//!     /// Production
//!     Prod,
//!     /// Certification
//!     Cert,
//!     /// Sandbox
//!     Sandbox,
//!     /// Local
//!     Local,
//! }
//!
//! impl fmt::Display for MyHierarchy {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         let env = match *self {
//!             MyHierarchy::Prod => "prod",
//!             MyHierarchy::Cert => "ce",
//!             MyHierarchy::Sandbox => "sb",
//!             MyHierarchy::Local => "local",
//!         };
//!         write!(f, "{}", env)
//!     }
//! }
//!
//! impl<'a> TryFrom<&'a str> for MyHierarchy {
//!     type Error = Error;
//!
//!     fn try_from(env: &str) -> Result<Self> {
//!         match env {
//!             "prod" => Ok(MyHierarchy::Prod),
//!             "ce" => Ok(MyHierarchy::Cert),
//!             "sb" => Ok(MyHierarchy::Sandbox),
//!             "local" => Ok(MyHierarchy::Local),
//!             _ => Err(Error::invalid_runtime_environment(env)),
//!         }
//!     }
//! }
//!
//! impl TryFrom<String> for MyHierarchy {
//!     type Error = Error;
//!
//!     fn try_from(env: String) -> Result<Self> {
//!         match &env[..] {
//!             "prod" => Ok(MyHierarchy::Prod),
//!             "ce" => Ok(MyHierarchy::Cert),
//!             "sb" => Ok(MyHierarchy::Sandbox),
//!             "local" => Ok(MyHierarchy::Local),
//!             _ => Err(Error::invalid_runtime_environment(&env)),
//!         }
//!     }
//! }
//!
//! impl Serialize for MyHierarchy {
//!     fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
//!     where
//!         S: Serializer,
//!     {
//!         serializer.serialize_str(&self.to_string())
//!     }
//! }
//!
//! impl<'de> Deserialize<'de> for MyHierarchy {
//!     fn deserialize<D>(deserializer: D) -> std::result::Result<MyHierarchy, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         struct MyHierarchyVisitor;
//!
//!         impl<'de> de::Visitor<'de> for MyHierarchyVisitor {
//!             type Value = MyHierarchy;
//!
//!             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//!                 formatter.write_str("any valid environment")
//!             }
//!
//!             fn visit_str<E>(self, value: &str) -> std::result::Result<MyHierarchy, E>
//!             where
//!                 E: de::Error,
//!             {
//!                 TryFrom::try_from(value).map_err(de::Error::custom)
//!             }
//!         }
//!
//!         deserializer.deserialize_string(MyHierarchyVisitor)
//!     }
//! }
//!
//! #[derive(Debug, De, Getters, Ser)]
//! struct MyAppEnv {
//!   /// The display name of this environment.
//!   #[get]
//!   name: String,
//!   /// The secret key only used in the Prod environment.
//!   #[get]
//!   key: Option<String>,
//! }
//! #
//! # fn foo() -> Result<()> {
//!
//! /// Grab your environment TOML.  This would usually be in a file and can
//! /// be read to a string such as below.
//! let toml = r#"[envs.prod]
//! name = "Production"
//! key = "abcd-123-efg-45"
//!
//! [envs.ce]
//! name = "Certification"
//!
//! [envs.sb]
//! name = "Sandbox"
//!
//! [envs.local]
//! name = "Local"
//! "#;
//!
//! // Deserialize the TOML config into your environment structs.  This example
//! // is using the custom `MyHierarchy` hierarchy.
//! let mut cursor = Cursor::new(toml);
//! let envs: Environments<MyHierarchy, MyAppEnv> = Environments::from_reader(&mut cursor)?;
//!
//! // Check the `Production` environment.
//! env::set_var("env", "prod");
//! let mut current = envs.current()?;
//! assert_eq!(current.name(), "Production");
//! assert_eq!(current.key(), &Some("abcd-123-efg-45".to_string()));
//!
//! // Switch to the `Sandbox` environment.
//! env::set_var("env", "sb");
//! current = envs.current()?;
//! assert_eq!(current.name(), "Sandbox");
//! assert_eq!(current.key(), &None);
//! #   Ok(())
//! # }
//!
//! fn main() {
//!     foo().unwrap()
//! }
//! ```
// rustc lints
#![cfg_attr(
    all(msrv, feature = "unstable", nightly),
    feature(
        c_unwind,
        lint_reasons,
        must_not_suspend,
        non_exhaustive_omitted_patterns_lint,
        strict_provenance
    )
)]
#![cfg_attr(
    msrv,
    deny(
        absolute_paths_not_starting_with_crate,
        anonymous_parameters,
        array_into_iter,
        asm_sub_register,
        bad_asm_style,
        bare_trait_objects,
        bindings_with_variant_name,
        // box_pointers,
        break_with_label_and_loop,
        clashing_extern_declarations,
        coherence_leak_check,
        confusable_idents,
        const_evaluatable_unchecked,
        const_item_mutation,
        dead_code,
        deprecated,
        deprecated_in_future,
        deprecated_where_clause_location,
        deref_into_dyn_supertrait,
        deref_nullptr,
        drop_bounds,
        duplicate_macro_attributes,
        dyn_drop,
        elided_lifetimes_in_paths,
        ellipsis_inclusive_range_patterns,
        explicit_outlives_requirements,
        exported_private_dependencies,
        forbidden_lint_groups,
        function_item_references,
        illegal_floating_point_literal_pattern,
        improper_ctypes,
        improper_ctypes_definitions,
        incomplete_features,
        indirect_structural_match,
        inline_no_sanitize,
        invalid_doc_attributes,
        invalid_value,
        irrefutable_let_patterns,
        keyword_idents,
        large_assignments,
        late_bound_lifetime_arguments,
        legacy_derive_helpers,
        let_underscore_drop,
        macro_use_extern_crate,
        meta_variable_misuse,
        missing_abi,
        missing_copy_implementations,
        missing_debug_implementations,
        missing_docs,
        mixed_script_confusables,
        named_arguments_used_positionally,
        no_mangle_generic_items,
        non_ascii_idents,
        non_camel_case_types,
        non_fmt_panics,
        non_shorthand_field_patterns,
        non_snake_case,
        non_upper_case_globals,
        nontrivial_structural_match,
        noop_method_call,
        overlapping_range_endpoints,
        path_statements,
        pointer_structural_match,
        private_in_public,
        redundant_semicolons,
        renamed_and_removed_lints,
        repr_transparent_external_private_fields,
        rust_2021_incompatible_closure_captures,
        rust_2021_incompatible_or_patterns,
        rust_2021_prefixes_incompatible_syntax,
        rust_2021_prelude_collisions,
        semicolon_in_expressions_from_macros,
        single_use_lifetimes,
        special_module_name,
        stable_features,
        suspicious_auto_trait_impls,
        temporary_cstring_as_ptr,
        trivial_bounds,
        trivial_casts,
        trivial_numeric_casts,
        type_alias_bounds,
        tyvar_behind_raw_pointer,
        uncommon_codepoints,
        unconditional_recursion,
        unexpected_cfgs,
        uninhabited_static,
        unknown_lints,
        unnameable_test_items,
        unreachable_code,
        unreachable_patterns,
        unreachable_pub,
        unsafe_code,
        unsafe_op_in_unsafe_fn,
        unstable_name_collisions,
        unstable_syntax_pre_expansion,
        unsupported_calling_conventions,
        unused_allocation,
        unused_assignments,
        unused_attributes,
        unused_braces,
        unused_comparisons,
        unused_crate_dependencies,
        unused_doc_comments,
        unused_extern_crates,
        unused_features,
        unused_import_braces,
        unused_imports,
        unused_labels,
        unused_lifetimes,
        unused_macro_rules,
        unused_macros,
        unused_must_use,
        unused_mut,
        unused_parens,
        unused_qualifications,
        unused_results,
        unused_tuple_struct_fields,
        unused_unsafe,
        unused_variables,
        variant_size_differences,
        where_clauses_object_safety,
        while_true,
))]
// If nightly and unstable, allow `unstable_features`
#![cfg_attr(all(msrv, feature = "unstable", nightly), allow(unstable_features))]
// The unstable features
#![cfg_attr(
    all(msrv, feature = "unstable", nightly),
    deny(
        ffi_unwind_calls,
        fuzzy_provenance_casts,
        lossy_provenance_casts,
        must_not_suspend,
        non_exhaustive_omitted_patterns,
        unfulfilled_lint_expectations,
    )
)]
// If nightly and not unstable, deny `unstable_features`
#![cfg_attr(all(msrv, not(feature = "unstable"), nightly), deny(unstable_features))]
// nightly only lints
// #![cfg_attr(all(msrv, nightly),deny())]
// nightly or beta only lints
#![cfg_attr(
    all(msrv, any(beta, nightly)),
    deny(for_loops_over_fallibles, opaque_hidden_inferred_bound)
)]
// beta only lints
// #![cfg_attr( all(msrv, beta), deny())]
// beta or stable only lints
// #![cfg_attr(all(msrv, any(beta, stable)), deny())]
// stable only lints
// #![cfg_attr(all(msrv, stable), deny())]
// clippy lints
#![cfg_attr(msrv, deny(clippy::all, clippy::pedantic))]
// #![cfg_attr(msrv, allow())]

mod env;
mod error;

pub use env::Environment;
pub use env::Environments;
pub use error::{Error, Result};
