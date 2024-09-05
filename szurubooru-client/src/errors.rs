//! Types that represent the various error states that can occur when interacting with
//! Szurubooru

use crate::models::SzuruEither;
use base64::EncodeSliceError;
use derive_builder::UninitializedFieldError;
#[cfg(feature = "python")]
use pyo3::{create_exception, exceptions::PyException, prelude::*};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;
use thiserror::Error;
use url::ParseError as UParseError;

/// Trait to support conversion into a [SzurubooruResult]
#[doc(hidden)]
pub trait IntoClientResult<T> {
    /// Convert `self` into a SzurubooruResult
    fn into_result(self) -> SzurubooruResult<T>;
}

#[derive(Debug, Error, AsRefStr)]
/// Type that represents the various error states that can occur when interacting with
/// Szurubooru
pub enum SzurubooruClientError {
    /// Error occurred when trying to Bas64 encode the `username:token` string
    #[error("Error encoding authentication token: {0}")]
    Base64EncodingError(#[source] EncodeSliceError),
    /// Error parsing the given host URL
    #[error("Error parsing URL {url}: {source}")]
    UrlParseError {
        /// The resulting error
        source: UParseError,
        /// The URL in question
        url: String,
    },
    /// Error occurred building the request before it's sent to the server
    #[error("Error building request {0}")]
    RequestBuilderError(#[source] reqwest::Error),
    /// Error occurred pas part of the request to the server
    #[error("Request error {0}")]
    RequestError(#[source] reqwest::Error),
    /// Error response with a text response from the server
    #[error("Response error {0}: Server reply: {1}")]
    ResponseError(StatusCode, String),
    /// Error parsing the JSON response from the server
    #[error("Response Parsing error: {0}: {1}")]
    ResponseParsingError(
        /// The JSON parsing error
        #[source]
        serde_json::Error,
        /// The string we attempted to parse
        String,
    ),
    /// Error serializing an object as JSON
    #[error("JSON Serialization error: {0}")]
    JSONSerializationError(#[source] serde_json::Error),
    /// Error when validation fails for one of the Builder types
    #[error("Validation error: {0}")]
    ValidationError(String),
    /// Error occurred when reading a file
    #[error("IO Error: {0}")]
    IOError(#[source] std::io::Error),
    /// Error returned by the Szurubooru server
    #[error("Error returned from Szurubooru host: {0:?}")]
    SzurubooruServerError(SzurubooruServerError),
}

impl From<SzurubooruServerError> for SzurubooruClientError {
    fn from(value: SzurubooruServerError) -> Self {
        SzurubooruClientError::SzurubooruServerError(value)
    }
}

impl From<UninitializedFieldError> for SzurubooruClientError {
    fn from(value: UninitializedFieldError) -> Self {
        SzurubooruClientError::ValidationError(value.to_string())
    }
}

#[cfg(feature = "python")]
create_exception!(
    szurubooru_client,
    SzuruClientError,
    PyException,
    "An exception that contains two pieces of information: The error kind and error details"
);

#[cfg(feature = "python")]
impl std::convert::From<SzurubooruClientError> for PyErr {
    fn from(value: SzurubooruClientError) -> Self {
        SzuruClientError::new_err((value.as_ref().to_string(), value.to_string()))
    }
}

/// Type used to represent success or a failure of some kind
pub type SzurubooruResult<T> = Result<T, SzurubooruClientError>;

#[doc(hidden)]
impl<T> IntoClientResult<T> for SzuruEither<T, SzurubooruServerError> {
    fn into_result(self) -> SzurubooruResult<T> {
        match self {
            SzuruEither::Left(v) => Ok(v),
            SzuruEither::Right(e) => Err(SzurubooruClientError::SzurubooruServerError(e)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
/// An error type returned by the server
pub enum SzurubooruServerErrorType {
    /// Inavlid pool category color
    InvalidPoolCategoryColorError,
    /// Missing required file
    MissingRequiredFileError,
    /// Missing required parameter
    MissingRequiredParameterError,
    /// Invalid parameter
    InvalidParameterError,
    /// Integrity error
    IntegrityError,
    /// Search error
    SearchError,
    /// Invalid authentication
    AuthError,
    /// Post with the given ID not found
    PostNotFoundError,
    /// Post is already featured
    PostAlreadyFeaturedError,
    /// Post is already uploaded
    PostAlreadyUploadedError,
    /// Invalid post ID
    InvalidPostIdError,
    /// Invalid Post Safety
    InvalidPostSafetyError,
    /// Invalid post source
    InvalidPostSourceError,
    /// Invalid post content
    InvalidPostContentError,
    /// Invalid post relation
    InvalidPostRelationError,
    /// Invalid post note
    InvalidPostNoteError,
    /// Invalid post flag
    InvalidPostFlagError,
    /// Invalid favorite target
    InvalidFavoriteTargetError,
    /// Invalid comment ID
    InvalidCommentIdError,
    /// Comment not found
    CommentNotFoundError,
    /// Empty comment text
    EmptyCommentTextError,
    /// Invalid score target
    InvalidScoreTargetError,
    /// Invalid score value
    InvalidScoreValueError,
    /// Tag category not found
    TagCategoryNotFoundError,
    /// Tag category already exists
    TagCategoryAlreadyExistsError,
    /// Tag category is in use
    TagCategoryIsInUseError,
    /// Invalid tag category name
    InvalidTagCategoryNameError,
    /// Invalid tag category color
    InvalidTagCategoryColorError,
    /// Tag not found
    TagNotFoundError,
    /// Tag already exists
    TagAlreadyExistsError,
    /// Tag is in use
    TagIsInUseError,
    /// Invalid tag name
    InvalidTagNameError,
    /// Invalid tag relation
    InvalidTagRelationError,
    /// Invalid tag category
    InvalidTagCategoryError,
    /// Invalid tag description
    InvalidTagDescriptionError,
    /// User not found
    UserNotFoundError,
    /// User already exists
    UserAlreadyExistsError,
    /// Invalid user name
    InvalidUserNameError,
    /// Invalid email
    InvalidEmailError,
    /// Invalid password
    InvalidPasswordError,
    /// Invalid rank
    InvalidRankError,
    /// Invalid avatar
    InvalidAvatarError,
    /// Processing error
    ProcessingError,
    /// Validation error
    ValidationError,
}

#[derive(Debug, Serialize, Deserialize)]
/// Type describing an error returned from Szurubooru
pub struct SzurubooruServerError {
    /// The name (or type) of error
    pub name: SzurubooruServerErrorType,
    /// Title of the error
    pub title: String,
    /// More of a description of the error
    pub description: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn test_parse_server_error() {
        let json_response = r#"{
        "name": "ValidationError",
        "title": "Validation Error",
        "description": "Some sort of validation error"
        }"#;

        let sse = serde_json::from_str::<SzurubooruServerError>(json_response)
            .expect("Failed to parse the JSON response");

        assert_eq!(sse.name, SzurubooruServerErrorType::ValidationError);
        assert_eq!(sse.title, "Validation Error");
        assert_eq!(sse.description, "Some sort of validation error");
    }
}
