//! SzurubooruClient is a wrapper around the excellently-documented Szurubooru API,
//! including type-safe (if not API-safe) Query and Sort tokens.
//!
//! # Creating a new client
//!
//! ## Basic authentication
//! Please keep in mind that this is not the preferred method of authentication. Tokens
//! are far superior.
//!
//! ```rust,no_run
//! use szurubooru_client::SzurubooruClient;
//! let client = SzurubooruClient::new_with_basic_auth("http://localhost:5001", "myuser",
//!     "mypassword", true).unwrap();
//! ```
//!
//! ## Token authentication
//! The far superior and more secure means of authentication
//!
//! ```rust,no_run
//! use szurubooru_client::SzurubooruClient;
//! let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
//! ```
//!
//! For all other methods for making the requests, see the documentation.
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

/// Core client module
pub mod client;
pub use client::SzurubooruClient;
pub use client::SzurubooruRequest;

pub mod errors;
pub use errors::SzurubooruResult;
pub mod models;
pub mod tokens;

#[cfg(feature = "python")]
#[doc(hidden)]
pub mod py;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymodule)]
/// A Python wrapper around SzurubooruClient
mod szurubooru_client {
    use pyo3::prelude::*;

    #[pymodule_export]
    pub use crate::{
        errors::SzuruClientError,
        /*models::{
            AroundPostResult, CommentResource, GlobalInfo, ImageSearchResult,
            ImageSearchSimilarPost, MicroPoolResource, MicroPostResource, MicroTagResource,
            MicroUserResource, NoteResource, PoolCategoryResource, PoolResource, PostResource,
            PostSafety, PostType, SnapshotCreationDeletionData, SnapshotData,
            SnapshotModificationData, SnapshotOperationType, SnapshotResource,
            SnapshotResourceType, TagCategoryResource, TagResource, TagSibling,
            UserAuthTokenResource, UserAvatarStyle, UserRank, UserResource,
        },
        tokens::{
            anonymous_token, named_token, sort_token, special_token, CommentNamedToken,
            CommentSortToken, PoolNamedToken, PoolSortToken, PostNamedToken, PostSortToken,
            PostSpecialToken, QueryToken, SnapshotNamedToken, TagNamedToken, TagSortToken,
            UserNamedToken, UserSortToken,
        },*/
        py::asynchronous::PythonAsyncClient, py::synchronous::PythonSyncClient,
        py::PyPagedSearchResult,
    };

    #[pymodule(name = "_tokens")]
    mod tokens {
        #[pymodule_export]
        pub use crate::tokens::{
            anonymous_token, named_token, sort_token, special_token, CommentNamedToken,
            CommentSortToken, PoolNamedToken, PoolSortToken, PostNamedToken, PostSortToken,
            PostSpecialToken, QueryToken, SnapshotNamedToken, TagNamedToken, TagSortToken,
            UserNamedToken, UserSortToken,
        };
    }

    #[pymodule(name = "_models")]
    mod models {
        #[pymodule_export]
        pub use crate::models::{
            AroundPostResult, CommentResource, GlobalInfo, ImageSearchResult,
            ImageSearchSimilarPost, MicroPoolResource, MicroPostResource, MicroTagResource,
            MicroUserResource, NoteResource, PoolCategoryResource, PoolResource, PostResource,
            PostSafety, PostType, SnapshotCreationDeletionData, SnapshotData,
            SnapshotModificationData, SnapshotOperationType, SnapshotResource,
            SnapshotResourceType, TagCategoryResource, TagResource, TagSibling,
            UserAuthTokenResource, UserAvatarStyle, UserRank, UserResource,
        };
    }
}
