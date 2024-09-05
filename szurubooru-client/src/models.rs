//! Types that represent the various API objects returned by Szurubooru. Many of the `Resource`
//! objects have all of their fields as [Option] types because the Server API supports field
//! selection.
//!
//! See [here](https://github.com/rr-/szurubooru/blob/master/doc/API.md#field-selecting) for
//! more information.

use crate::errors::SzurubooruClientError;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::AsRefStr;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use serde_pyobject::to_pyobject;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
/// Enum used to represent something that's either `Left` or `Right`
pub enum SzuruEither<L, R> {
    /// Enum variant `Left`
    Left(L),
    /// Enum variant `Right`
    Right(R),
}

#[derive(Debug, Serialize, Deserialize)]
/// A result of search operation that doesn't involve paging
pub struct UnpagedSearchResult<T> {
    /// The total list of results
    pub results: Vec<T>,
}

impl<T: WithBaseURL> WithBaseURL for UnpagedSearchResult<T> {
    fn with_base_url(self, url: &str) -> Self {
        Self {
            results: self.results.with_base_url(url),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// A result of search operation that involves paging
///
/// Use [offset](crate::SzurubooruRequest::with_offset) and [limit](crate::SzurubooruRequest::with_limit)
/// to fetch the next page
pub struct PagedSearchResult<T> {
    /// The original query for the request
    pub query: String,
    /// The number of `T` to skip forward
    pub offset: u32,
    /// The maximum number of `T` to return
    pub limit: u32,
    /// The total number of `T` that match the [query](PagedSearchResult::query)
    pub total: u32,
    /// The results themselves
    pub results: Vec<T>,
}

impl<T: WithBaseURL> WithBaseURL for PagedSearchResult<T> {
    fn with_base_url(self, url: &str) -> Self {
        Self {
            results: self.results.with_base_url(url),
            ..self
        }
    }
}

pub(crate) trait WithBaseURL {
    fn with_base_url(self, url: &str) -> Self;
}

impl<T: WithBaseURL> WithBaseURL for Option<T> {
    fn with_base_url(self, url: &str) -> Self {
        self.map(|inner| inner.with_base_url(url))
    }
}

impl<T: WithBaseURL> WithBaseURL for Vec<T> {
    fn with_base_url(self, url: &str) -> Self {
        self.into_iter()
            .map(|inner| inner.with_base_url(url))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, eq, module = "szurubooru_client.models")
)]
/// A [tag resource](TagResource) stripped down to `names`, `category` and `usages` fields.
pub struct MicroTagResource {
    /// The tag names and aliases
    pub names: Vec<String>,
    /// The category this tag belongs to
    pub category: String,
    /// The number of times this tag has been used
    pub usages: u32,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl MicroTagResource {
    /// Function that generates the representation string for this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// To prevent problems with concurrent resource modification, Szurubooru implements optimistic
/// locks using resource versions. Each modifiable resource has its version returned to the client
/// with `GET` requests. At the same time, each `PUT` and `DELETE` request sent by the client
/// must present the same version field to the server with value as it was given in `GET`.
///
/// For example, given `GET /post/1`, the server responds like this:
///
/// ```json
/// {
///     ...,
///     "version": 2
/// }
/// ```
///
/// This means the client must then send `{"version": 2}` back too. If the client fails to do so,
/// the server will reject the request notifying about missing parameter. If someone has edited the
/// post in the meantime, the server will reject the request as well, in which case the client is
/// encouraged to notify the user about the situation.
pub struct ResourceVersion {
    /// The version itself
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
/// A single tag. Tags are used to let users search for posts.
pub struct TagResource {
    /// resource version. See [versioning](ResourceVersion)
    pub version: u32,
    /// a list of tag names (aliases). Tagging a post with any name will automatically assign
    /// the first name from this list.
    pub names: Option<Vec<String>>,
    /// the name of the category the given tag belongs to
    pub category: Option<String>,
    /// a list of implied tags, serialized as micro tag resource. Implied tags are automatically
    /// appended by the web client on usage.
    pub implications: Option<Vec<MicroTagResource>>,
    /// a list of suggested tags, serialized as micro tag resource. Suggested tags are shown to
    /// the user by the web client on usage
    pub suggestions: Option<Vec<MicroTagResource>>,
    /// time the tag was created
    pub creation_time: Option<DateTime<Utc>>,
    /// time the tag was edited
    pub last_edit_time: Option<DateTime<Utc>>,
    /// the number of posts the tag was used in
    pub usages: Option<u32>,
    /// the tag description (instructions how to use, history etc.) The client should render
    /// is as Markdown
    pub description: Option<String>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl TagResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

/// Creates or updates a tag using specified parameters. Names, suggestions and implications must
/// match `tag_name_regex` from server's configuration. Category must exist and is the same as name
/// field within [TagCategoryResource] resource. Suggestions and implications are optional. If specified
/// implied tags or suggested tags do not exist yet, they will be automatically created. Tags
/// created automatically have no implications, no suggestions, one name and their category is set
/// to the first tag category found. If there are no tag categories established yet, an error
/// will be thrown.
///
/// ```no_run
/// use szurubooru_client::models::CreateUpdateTagBuilder;
/// let cu_tag = CreateUpdateTagBuilder::default()
///                 .version(1)
///                 .names(vec!["foo_tag".to_string()])
///                 .build()
///                 .expect("A new tag");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
//#[builder(pattern="owned")]
#[builder(setter(strip_option), build_fn(error = "SzurubooruClientError"))]
pub struct CreateUpdateTag {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    /// resource version. See [versioning](ResourceVersion)
    pub version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    /// Tag names and aliases, must match `tag_name_regex` from the server's configuration
    pub names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    /// Category that this tag belongs to. Must already exist
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    /// The tag description in Markdown format
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    /// Tags that should be implied when this tag is used
    pub implications: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    /// Tags that should be suggested when this tag is used
    pub suggestions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
/// A single tag category. The primary purpose of tag categories is to distinguish certain tag
/// types (such as characters, media type etc.), which improves user experience.
pub struct TagCategoryResource {
    /// resource version. See [versioning](ResourceVersion)
    pub version: u32,
    /// The name of the tag category
    pub name: Option<String>,
    /// The display color of the tag category
    pub color: Option<String>,
    /// How many tags is the given category used with
    pub usages: Option<u32>,
    /// The order in which tags with this category are displayed, ascending
    pub order: Option<u32>,
    /// Whether the tag category is the default one
    pub default: Option<bool>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl TagCategoryResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
#[builder(setter(strip_option), build_fn(error = "SzurubooruClientError"))]

/// Used for creating or updating a Tag Category
pub struct CreateUpdateTagCategory {
    /// Resource version. See [versioning](ResourceVersion)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub version: Option<u32>,
    /// The name of the category to create
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub name: Option<String>,
    /// The display color to use for the category
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub color: Option<String>,
    /// The order in which tags with this category are displayed, ascending
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub order: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(strip_option), build_fn(error = "SzurubooruClientError"))]
#[serde(rename_all = "camelCase")]
/// Removes source tag and merges all of its usages, suggestions and implications to the target tag.
/// Other tag properties such as category and aliases do not get transferred and are discarded.
pub struct MergeTags {
    /// Version of the tag to remove
    #[serde(rename = "removeVersion")]
    pub remove_tag_version: u32,
    /// The name of the tag to remove
    #[serde(rename = "remove")]
    pub remove_tag: String,
    /// The version of the tag to merge TO
    pub merge_to_version: u32,
    /// The name of the tag to merge TO
    #[serde(rename = "mergeTo")]
    pub merge_to_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
/// Lists siblings of given tag, e.g. tags that were used in the same posts as the given tag
pub struct TagSibling {
    /// The related tag
    pub tag: TagResource,
    /// How many times a given tag appears with the given tag
    pub occurrences: u32,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl TagSibling {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, eq_int, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// The type of post
pub enum PostType {
    /// Image post
    Image,
    /// Animated post
    Animation,
    /// Alias of [Animation](PostType::Animation)
    Animated,
    /// Alias of [Animation](PostType::Animation)
    Anim,
    /// Flash animation
    Flash,
    /// Alias of [Flash](PostType::Flash)
    Swf,
    /// Video post of some type. See the mime type for more information
    Video,
    /// Webm container type
    Webm,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, eq_int, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// How SFW/NSFW the post is
pub enum PostSafety {
    /// Post is SFW
    Safe,
    /// Post is possibly NSFW
    Sketchy,
    /// Alias of [Sketchy](PostSafety::Sketchy)
    Questionable,
    /// Post is NSFW
    Unsafe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A post resource stripped down to `id` and `thumbnailUrl` fields.
pub struct MicroPostResource {
    /// The ID of the post
    pub id: u32,
    /// The thumbnail URL of the post
    pub thumbnail_url: String,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl MicroPostResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for MicroPostResource {
    fn with_base_url(self, url: &str) -> Self {
        if !self.thumbnail_url.contains(url) {
            MicroPostResource {
                id: self.id,
                thumbnail_url: format!("{}{}", url, self.thumbnail_url),
            }
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[doc(hidden)]
pub(crate) struct PostId {
    pub id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A post resource
pub struct PostResource {
    /// Resource version. See [versioning](ResourceVersion)
    pub version: Option<u32>,
    /// The post identifier
    pub id: Option<u32>,
    /// Time the post was created
    pub creation_time: Option<DateTime<Utc>>,
    /// Time the post was edited
    pub last_edit_time: Option<DateTime<Utc>>,
    /// Whether the post is safe for work
    pub safety: Option<PostSafety>,
    #[serde(rename = "type")]
    /// The type of the post
    pub post_type: Option<PostType>,
    /// Where the post was grabbed form, supplied by the user
    pub source: Option<String>,
    /// The SHA1 file checksum. Used in snapshots to signify changes of the post content
    pub checksum: Option<String>,
    #[serde(rename = "checksumMD5")]
    /// The MD5 file checksum
    pub checksum_md5: Option<String>,
    /// The original width of the post content.
    pub canvas_width: Option<u32>,
    /// The original height of the post content.
    pub canvas_height: Option<u32>,
    /// Where the post content is located
    pub content_url: Option<String>,
    /// Where the post thumbnail is located
    pub thumbnail_url: Option<String>,
    /// Various flags such as whether the post is looped
    pub flags: Option<Vec<String>>,
    /// List of tags the post is tagged with
    pub tags: Option<Vec<MicroTagResource>>,
    /// A list of related posts.
    pub relations: Option<Vec<MicroPostResource>>,
    /// A list of post annotations
    pub notes: Option<Vec<NoteResource>>,
    /// Who created the post
    pub user: Option<MicroUserResource>,
    /// The collective score (+1/-1 rating) of the given post
    pub score: Option<i32>,
    /// The user's score for this post
    pub own_score: Option<i32>,
    /// Whether the authenticated user has given post in their favorites
    pub own_favorite: Option<bool>,
    /// How many tags the post is tagged with
    pub tag_count: Option<u32>,
    /// How many users have the post in their favorites
    pub favorite_count: Option<u32>,
    /// How many comments are filed under that post
    pub comment_count: Option<u32>,
    /// How many notes the post has
    pub note_count: Option<u32>,
    /// How many times has the post been featured
    pub feature_count: Option<u32>,
    /// How many posts are related to this post
    pub relation_count: Option<u32>,
    /// The last time the post was featured
    pub last_feature_time: Option<DateTime<Utc>>,
    /// List of users who have favorited this post
    pub favorited_by: Option<Vec<MicroUserResource>>,
    /// Whether the post uses custom thumbnail
    pub has_custom_thumbnail: Option<bool>,
    /// Subsidiary to [type](PostResource::post_type), used to tell exact content format;
    /// useful for `<video>` tags for instance
    pub mime_type: Option<String>,
    /// All the comments on the post
    pub comment: Option<Vec<CommentResource>>,
    /// The pools in which the post is a member
    pub pools: Option<Vec<PoolResource>>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl PostResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for PostResource {
    fn with_base_url(self, url: &str) -> Self {
        let curl = self.content_url.map(|cu| {
            if !cu.contains(url) {
                format!("{}{}", url, cu)
            } else {
                cu
            }
        });
        let turl = self.thumbnail_url.map(|tu| {
            if !tu.contains(url) {
                format!("{}{}", url, tu)
            } else {
                tu
            }
        });

        let user = self.user.with_base_url(url);
        let relations = self.relations.with_base_url(url);
        let fv_by = self.favorited_by.with_base_url(url);
        let pools = self.pools.with_base_url(url);

        PostResource {
            content_url: curl,
            thumbnail_url: turl,
            user,
            relations,
            favorited_by: fv_by,
            pools,
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(strip_option), build_fn(error = "SzurubooruClientError"))]
#[serde(rename_all = "camelCase")]
/// A `struct` used to create or update a post. For updating purposes
/// the [version](CreateUpdatePost::version) field is required
pub struct CreateUpdatePost {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Resource version. See [versioning](ResourceVersion)
    #[builder(default)]
    pub version: Option<u32>,
    /// Tags to use for this post. If specified tags do not exist yet, they will be automatically
    /// created. Tags created automatically have no implications, no suggestions, one name and
    /// their category is set to the first tag category found
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Required field, represents the SFW/NSFW state of a post
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety: Option<PostSafety>,
    /// The origin of the post's content
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The IDs of related posts
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<u32>>,
    /// Notes to be displayed on the post
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<NoteResource>>,
    /// Flags relevant to the post. If omitted they will be auto-detected
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The URL to download the content from
    #[builder(default)]
    pub content_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The token returned from
    /// [upload_temporary_file](crate::SzurubooruRequest::upload_temporary_file)
    #[builder(default)]
    pub content_token: Option<String>,
    /// Upload the post anonymously
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymous: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A token representing a temporary file upload
pub struct TemporaryFileUpload {
    /// Temporary upload token
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(build_fn(error = "SzurubooruClientError"))]
#[serde(rename_all = "camelCase")]
/// Removes source post and merges all of its tags, relations, scores, favorites and comments to
/// the target post. If replaceContent is set to true, content of the target post is replaced using
/// the content of the source post; otherwise it remains unchanged. Source post properties such as
/// its safety, source, whether to loop the video and other scalar values do not get transferred
/// and are discarded.
pub struct MergePost {
    /// The version of the post to remove
    #[serde(rename = "removeVersion")]
    pub remove_post_version: u32,
    /// The ID of the post to remove
    #[serde(rename = "remove")]
    pub remove_post: u32,
    /// The version of the post to merge TO
    pub merge_to_version: u32,
    /// The post ID of the post to merge TO
    #[serde(rename = "mergeTo")]
    pub merge_to_post: u32,
    /// Whether to replace the content
    #[serde(rename = "replaceContent")]
    pub replace_post_content: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[doc(hidden)]
pub struct RateResource {
    pub score: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A text annotation rendered on top of the post
pub struct NoteResource {
    /// Where to draw the annotation. Each point must have coordinates within 0 to 1.
    /// For example, `[[0,0],[0,1],[1,1],[1,0]]` will draw the annotation on the whole post,
    /// whereas `[[0,0],[0,0.5],[0.5,0.5],[0.5,0]]` will draw it inside the post's upper left
    /// quarter
    pub polygon: Vec<Vec<u8>>,
    /// The annotation text, in Markdown format
    pub text: String,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl NoteResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, eq_int, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// The Rank of a given User
pub enum UserRank {
    /// Restricted, limited user
    Restricted,
    /// Regular user
    Regular,
    /// Power user
    Power,
    /// Moderator user
    Moderator,
    /// All-powerful Administrator
    Administrator,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, eq_int, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// The kind of User Avatar
pub enum UserAvatarStyle {
    /// Automatically-generated Gravatar
    Gravatar,
    /// Manually updated avatar
    Manual,
}

// Because pyo3 get_all doesn't let you exclude fields we have to define the fields twice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "python"), pyclass(module = "szurubooru_client.models"))]
#[serde(rename_all = "camelCase")]
/// A single user
pub struct UserResource {
    /// Resource version. See [versioning](ResourceVersion)
    #[cfg(feature = "python")]
    #[pyo3(get)]
    pub version: Option<u32>,

    /// Resource version. See [versioning](ResourceVersion)
    #[cfg(not(feature = "python"))]
    pub version: Option<u32>,

    /// The user's username
    #[cfg(feature = "python")]
    #[pyo3(get)]
    pub name: Option<String>,

    /// The user's username
    #[cfg(not(feature = "python"))]
    pub name: Option<String>,

    /// The user email. It is available only if the request is authenticated by the same user,
    /// or the authenticated user can change the email. If it's unavailable, the server returns
    /// `false`. If the user hasn't specified an email, the server returns [None](Option::None)
    pub email: Option<SzuruEither<String, bool>>,

    /// The user rank, which effectively affects their privileges
    #[cfg(feature = "python")]
    #[pyo3(get)]
    pub rank: Option<UserRank>,

    /// The user rank, which effectively affects their privileges
    #[cfg(not(feature = "python"))]
    pub rank: Option<UserRank>,

    /// The last login time
    #[cfg(feature = "python")]
    #[pyo3(get)]
    #[serde(rename = "last-login-time")]
    pub last_login_time: Option<DateTime<Utc>>,

    /// The last login time
    #[cfg(not(feature = "python"))]
    #[serde(rename = "last-login-time")]
    pub last_login_time: Option<DateTime<Utc>>,

    /// The user registration time
    #[serde(rename = "creation-time")]
    #[cfg(feature = "python")]
    #[pyo3(get)]
    pub creation_time: Option<DateTime<Utc>>,

    /// The user registration time
    #[serde(rename = "creation-time")]
    #[cfg(not(feature = "python"))]
    pub creation_time: Option<DateTime<Utc>>,

    /// How to render the user avatar
    #[cfg(feature = "python")]
    #[pyo3(get)]
    pub avatar_style: Option<UserAvatarStyle>,

    /// How to render the user avatar
    #[cfg(not(feature = "python"))]
    pub avatar_style: Option<UserAvatarStyle>,

    /// The URL to the avatar
    #[cfg(feature = "python")]
    #[pyo3(get)]
    pub avatar_url: Option<String>,

    /// The URL to the avatar
    #[cfg(not(feature = "python"))]
    pub avatar_url: Option<String>,

    /// Number of comments
    #[cfg(feature = "python")]
    #[pyo3(get)]
    #[serde(rename = "comment-count")]
    pub comment_count: Option<u32>,

    /// Number of comments
    #[cfg(not(feature = "python"))]
    #[serde(rename = "comment-count")]
    pub comment_count: Option<u32>,

    /// Number of uploaded posts
    #[cfg(feature = "python")]
    #[pyo3(get)]
    #[serde(rename = "uploaded-post-count")]
    pub uploaded_post_count: Option<u32>,

    /// Number of uploaded posts
    #[cfg(not(feature = "python"))]
    #[serde(rename = "uploaded-post-count")]
    pub uploaded_post_count: Option<u32>,

    /// Number of liked posts. It is available only if the request is authenticated by the same
    /// user. If it's unavailable, the server returns `false`
    #[serde(rename = "liked-post-count")]
    pub liked_post_count: Option<SzuruEither<u32, bool>>,

    /// Number of disliked posts. It is available only if the request is authenticated by the same
    /// user. If it's unavailable, the server returns `false`.
    #[serde(rename = "disliked-post-count")]
    pub disliked_post_count: Option<SzuruEither<u32, bool>>,

    /// Number of favorited posts
    #[serde(rename = "favorite-post-count")]
    pub favorite_post_count: Option<SzuruEither<u32, bool>>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl UserResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    #[getter]
    #[pyo3(name = "email")]
    /// Returns this resource's email field, if the current user has permission to see it
    pub fn email_py(&self) -> PyResult<Option<String>> {
        match &self.email {
            None => Ok(None),
            Some(SzuruEither::Left(s)) => Ok(Some(s.to_string())),
            Some(SzuruEither::Right(_)) => Ok(None),
        }
    }

    #[getter]
    #[pyo3(name = "liked_post_count")]
    /// Returns this resource's liked_post_count, if the current user has permission to see it
    pub fn liked_post_count_py(&self) -> PyResult<Option<u32>> {
        match &self.liked_post_count {
            None => Ok(None),
            Some(SzuruEither::Left(s)) => Ok(Some(*s)),
            Some(SzuruEither::Right(_)) => Ok(None),
        }
    }

    #[getter]
    #[pyo3(name = "disliked_post_count")]
    /// Returns this resource's disliked_post_count, if the current user has permission to see it
    pub fn disliked_post_count_py(&self) -> PyResult<Option<u32>> {
        match &self.disliked_post_count {
            None => Ok(None),
            Some(SzuruEither::Left(s)) => Ok(Some(*s)),
            Some(SzuruEither::Right(_)) => Ok(None),
        }
    }

    #[getter]
    #[pyo3(name = "favorite_post_count")]
    /// Returns this resource's favorite_post_count, if the current user has permission to see it
    pub fn favorite_post_count_py(&self) -> PyResult<Option<u32>> {
        match &self.favorite_post_count {
            None => Ok(None),
            Some(SzuruEither::Left(s)) => Ok(Some(*s)),
            Some(SzuruEither::Right(_)) => Ok(None),
        }
    }
}

impl WithBaseURL for UserResource {
    fn with_base_url(self, url: &str) -> Self {
        let av_url = self.avatar_url.map(|au| {
            if !au.contains(url) {
                format!("{}{}", url, au)
            } else {
                au
            }
        });
        UserResource {
            avatar_url: av_url,
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
#[builder(setter(strip_option), build_fn(error = "SzurubooruClientError"))]
#[serde(rename_all = "camelCase")]
/// `struct` used to create or update a user resource. The version field is only used when
/// updating an existing resource
pub struct CreateUpdateUser {
    /// Resource version. See [versioning](ResourceVersion)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub version: Option<u32>,
    /// The username
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The user's password
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// The user's desired rank, if not given will default to `default_rank` in the server's
    /// configuration
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<UserRank>,
    /// The user avatar style, Gravatar or Manual
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_style: Option<UserAvatarStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A user resource stripped down to `name` and `avatarUrl` fields
pub struct MicroUserResource {
    /// The username
    pub name: String,
    /// The user's avatar URL
    pub avatar_url: String,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl MicroUserResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for MicroUserResource {
    fn with_base_url(self, url: &str) -> Self {
        if !self.avatar_url.contains(url) {
            MicroUserResource {
                name: self.name,
                avatar_url: format!("{}{}", url, self.avatar_url),
            }
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "kebab-case")]
/// A single user token
pub struct UserAuthTokenResource {
    /// A micro user resource
    pub user: Option<MicroUserResource>,
    /// The token that can be used to authenticate the user.
    pub token: Option<String>,
    /// A note that describes the token
    pub note: Option<String>,
    /// Whether the token is still valid for authentication
    pub enabled: Option<bool>,
    /// Time when the token expires
    pub expiration_time: Option<DateTime<Utc>>,
    /// Resource version. See [versioning](ResourceVersion)
    pub version: Option<u32>,
    /// time the user token was created
    pub creation_time: Option<DateTime<Utc>>,
    /// time the user token was edited
    pub last_edit_time: Option<DateTime<Utc>>,
    /// the last time this token was used
    pub last_usage_time: Option<DateTime<Utc>>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl UserAuthTokenResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for UserAuthTokenResource {
    fn with_base_url(self, url: &str) -> Self {
        Self {
            user: self.user.with_base_url(url),
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(setter(into, strip_option), build_fn(error = "SzurubooruClientError"))]
#[serde(rename_all = "kebab-case")]
/// `struct` to create or update a UserAuthToken. `version` is only required when updating an
/// existing resource
pub struct CreateUpdateUserAuthToken {
    /// Resource version. See [versioning](ResourceVersion)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub version: Option<u32>,
    /// Whether the token is still valid for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub enabled: Option<bool>,
    /// A note that describes the token
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub note: Option<String>,
    /// Time when the token expires
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub expiration_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct PasswordResetToken {
    /// The password token received via email
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Type that represents a new temporary password
pub struct TemporaryPassword {
    /// The new temporary password generated once [PasswordResetToken] has been sent to the server
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// Simple server configuration
pub struct GlobalInfoConfig {
    /// Regular expression that the username must match
    pub user_name_regex: String,
    /// Regular expression that the password must match
    pub password_regex: String,
    /// Regular expression that tag names must match
    pub tag_name_regex: String,
    /// Regular expression that tag category names must match
    pub tag_category_name_regex: String,
    /// Default user rank upon signup
    pub default_user_rank: String,
    /// Whether safety is enabled
    pub enable_safety: bool,
    /// Contact email for this server
    pub contact_email: Option<String>,
    /// Is sending email enabled for this server
    pub can_send_mails: bool,
    /// Available privileges enabled for this server
    pub privileges: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// Simple server statistics
pub struct GlobalInfo {
    /// The total number of posts
    pub post_count: u32,
    /// Total disk usage
    pub disk_usage: u32,
    /// The current featured post
    pub featured_post: Option<u32>,
    /// The time the current featured post was featured
    pub featuring_time: Option<DateTime<Utc>>,
    /// The user who uploaded the featured post
    pub featuring_user: Option<u32>,
    /// The current server time
    pub server_time: DateTime<Utc>,
    /// The configuration for this server
    pub config: GlobalInfoConfig,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl GlobalInfo {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A single pool category. The primary purpose of pool categories is to distinguish certain pool
/// types (such as series, relations etc.), which improves user experience.
pub struct PoolCategoryResource {
    /// Resource version. See [versioning](ResourceVersion)
    pub version: Option<u32>,
    /// The category name
    pub name: Option<String>,
    /// The category color
    pub color: Option<String>,
    /// How many pools is the given category used with
    pub usages: Option<u32>,
    /// Whether the pool category is the default one
    pub default: Option<bool>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl PoolCategoryResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(strip_option), build_fn(error = "SzurubooruClientError"))]
/// `struct` used for creating or updating a pool category. This type uses a Builder pattern like
/// so:
///
/// ```no_run
/// use szurubooru_client::models::CreateUpdatePoolCategoryBuilder;
/// // Updating an existing pool category
/// let cu_pool_cat = CreateUpdatePoolCategoryBuilder::default()
///                         .version(1)
///                         .name("new_name".to_string())
///                         .build()
///                         .unwrap();
/// ```
pub struct CreateUpdatePoolCategory {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    /// Category version (used for updating)
    pub version: Option<u32>,
    /// Category name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub name: Option<String>,
    /// Category color
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// Type that represents a Pool resource
pub struct PoolResource {
    /// Resource version. See [versioning](ResourceVersion)
    pub version: Option<u32>,
    /// The pool identifier
    pub id: Option<u32>,
    /// A list of pool names (aliases)
    pub names: Option<Vec<String>>,
    /// The name of the category the given pool belongs to
    pub category: Option<String>,
    /// An ordered list of posts. Posts are ordered by insertion by default
    pub posts: Option<Vec<MicroPostResource>>,
    /// Time the pool was created
    pub creation_time: Option<DateTime<Utc>>,
    /// Time the pool was edited
    pub last_edit_time: Option<DateTime<Utc>>,
    /// The total number of posts the pool has
    pub post_count: Option<u32>,
    /// The pool description (instructions how to use, history etc). The client should render
    /// it as Markdown
    pub description: Option<String>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl PoolResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for PoolResource {
    fn with_base_url(self, url: &str) -> Self {
        PoolResource {
            posts: self.posts.with_base_url(url),
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(setter(strip_option), build_fn(error = "SzurubooruClientError"))]
#[serde(rename_all = "camelCase")]
/// This type is used when creating or updating a pool object. It uses the builder pattern like so:
///
/// ```no_run
/// use szurubooru_client::models::CreateUpdatePoolBuilder;
/// // Create a new pool
/// let create_pool = CreateUpdatePoolBuilder::default()
///                     .names(vec!["foo".to_string(), "bar".to_string()])
///                     .description("Markdown string".to_string())
///                     .build()
///                     .unwrap();
/// ```
///
pub struct CreateUpdatePool {
    /// Resource version. See [versioning](ResourceVersion)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub version: Option<u32>,
    /// Names and aliases for this pool. When creating a new pool the first name in this list
    /// is used as the pool name
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    /// Pool category that this pool belongs to. Must already exist
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Markdown string describing this pool
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A list of posts that belong to this pool. The server will throw an error if one of these
    /// post IDs doesn't exist
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub posts: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(build_fn(error = "SzurubooruClientError"))]
#[serde(rename_all = "camelCase")]
/// This type is used to specify which pools should be merged. Uses the builder pattern like so:
///
/// ```no_run
/// use szurubooru_client::models::MergePoolBuilder;
/// // Merge pool ID 1 at version 1 to pool ID 3 at version 5
/// let merge_pool = MergePoolBuilder::default()
///                     .remove_pool_version(1)
///                     .remove_pool(1)
///                     .merge_to_version(5)
///                     .merge_to_pool(3)
///                     .build()
///                     .unwrap();
/// ```
pub struct MergePool {
    /// Version of the pool to remove. Must match the current Pool version
    #[serde(rename = "removeVersion")]
    pub remove_pool_version: u32,
    /// Pool ID to remove
    #[serde(rename = "remove")]
    pub remove_pool: u32,
    /// Version of the pool to merge TO
    pub merge_to_version: u32,
    /// Pool ID of the pool to merge TO
    #[serde(rename = "mergeTo")]
    pub merge_to_pool: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A micro resource representing a Pool. A subset of the fields of a [PoolResource].
pub struct MicroPoolResource {
    /// The pool ID
    pub id: Option<u32>,
    /// Name and aliases for this pool
    pub names: Option<Vec<String>>,
    /// The category this pool belongs to
    pub category: Option<String>,
    /// The total number of posts in this pool
    pub post_count: Option<u32>,
    /// A markdown string describing the pool
    pub description: Option<String>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl MicroPoolResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A type representing a Comment on a post
pub struct CommentResource {
    /// Resource version. See [versioning](ResourceVersion)
    pub version: Option<u32>,
    /// The comment ID
    pub id: Option<u32>,
    /// The post ID this comment belongs to
    pub post_id: Option<u32>,
    /// The user who had posted this comment
    pub user: Option<MicroUserResource>,
    /// The text of the comment
    pub text: Option<String>,
    /// When was the comment posted
    pub creation_time: Option<DateTime<Utc>>,
    /// When was the last time this comment was edited
    pub last_edit_time: Option<DateTime<Utc>>,
    /// The sum of the -1/0/+1 scores by other users
    pub score: Option<i32>,
    /// The user's own score for this comment
    pub own_score: Option<i32>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl CommentResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(setter(strip_option), build_fn(error = "SzurubooruClientError"))]
#[serde(rename_all = "camelCase")]
/// This type is used when creating or updating a comment. This type uses the builder pattern like
/// so:
///
/// ```no_run
/// use szurubooru_client::models::CreateUpdateCommentBuilder;
/// // Update an existing comment's text for the post ID 1234
/// let update_comment = CreateUpdateCommentBuilder::default()
///                         .version(1)
///                         .text("Hello this is my comment".to_string())
///                         .post_id(1234)
///                         .build()
///                         .unwrap();
/// ```
pub struct CreateUpdateComment {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Resource version. See [versioning](ResourceVersion)
    /// Omitted when creating a new comment
    pub version: Option<u32>,
    /// The text of the comment
    pub text: String,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The post the comment should be attached to. Only used when creating a new comment
    pub post_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, eq_int, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// The kind of snapshot that has been recorded
pub enum SnapshotOperationType {
    /// Item was created
    Created,
    /// Item was modified
    Modified,
    /// Item was deleted
    Deleted,
    /// Item was merged
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, eq_int, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// The kind of resource described by this snapshot
pub enum SnapshotResourceType {
    /// Tag resource
    Tag,
    /// Tag category resource
    #[serde(rename = "tag_category")]
    TagCategory,
    /// Post resource
    Post,
    /// Pool resource
    Pool,
    /// Pool Category
    #[serde(rename = "pool_category")]
    PoolCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase", untagged)]
/// Data for a resource that was created
#[allow(clippy::large_enum_variant)]
pub enum SnapshotCreationDeletionData {
    /// A tag resource that was created
    Tag(TagResource),
    /// A tag category resource that was created
    TagCategory(TagCategoryResource),
    /// A post resource that was created
    Post(PostResource),
    /// A pool resource that was created
    Pool(PoolResource),
    /// A pool category resource that was created
    PoolCategory(PoolCategoryResource),
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl SnapshotCreationDeletionData {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for SnapshotCreationDeletionData {
    fn with_base_url(self, url: &str) -> Self {
        match self {
            SnapshotCreationDeletionData::Pool(pool) => {
                SnapshotCreationDeletionData::Pool(pool.with_base_url(url))
            }
            SnapshotCreationDeletionData::Post(post) => {
                SnapshotCreationDeletionData::Post(post.with_base_url(url))
            }
            _ => self,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// Data for a modified resource
pub struct SnapshotModificationData {
    #[cfg(feature = "python")]
    #[serde(rename = "type")]
    #[pyo3(get)]
    /// The type of snapshot
    pub snapshot_type: String,

    #[cfg(not(feature = "python"))]
    #[serde(rename = "type")]
    /// The type of snapshot
    pub snapshot_type: String,

    /// The JSON value for the modified resource. A dictionary diff that depends on the resource
    /// kind.
    ///
    /// See [here](https://github.com/rr-/szurubooru/blob/master/doc/API.md#snapshot) for more
    /// information
    pub value: serde_json::Value,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl SnapshotModificationData {
    #[getter]
    /// Get the value associated with this snapshot
    pub fn get_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let obj = to_pyobject(py, &self.value).unwrap().unbind();
        Ok(obj)
    }

    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(eq, module = "szurubooru_client.models")
)]
#[serde(untagged)]
/// Type representing the data as part of a snapshot
#[allow(clippy::large_enum_variant)]
pub enum SnapshotData {
    /// Data for a Created or Deleted resource
    CreateOrDelete(SnapshotCreationDeletionData),
    /// Data for a modified resource
    Modify(SnapshotModificationData),
    /// Data for a merged resource
    Merge(Vec<String>),
}

impl WithBaseURL for SnapshotData {
    fn with_base_url(self, url: &str) -> Self {
        match self {
            SnapshotData::CreateOrDelete(cod) => {
                SnapshotData::CreateOrDelete(cod.with_base_url(url))
            }
            _ => self,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// Overall type representing some sort of change to a resource
pub struct SnapshotResource {
    /// The operation type
    pub operation: Option<SnapshotOperationType>,
    #[serde(rename = "type")]
    /// The resource type
    pub resource_type: Option<SnapshotResourceType>,
    /// The ID of the snapshot itself
    pub id: Option<String>,
    /// The user who created this change
    pub user: Option<MicroUserResource>,
    /// The data associated with this resource change
    pub data: Option<SnapshotData>,
    /// When this resource change occurred
    pub time: Option<DateTime<Utc>>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl SnapshotResource {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for SnapshotResource {
    fn with_base_url(self, url: &str) -> Self {
        SnapshotResource {
            user: self.user.with_base_url(url),
            data: self.data.with_base_url(url),
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A result when searching for similar posts to a given image
pub struct ImageSearchSimilarPost {
    /// How close the post is to the given image
    pub distance: f32,
    /// The post in question
    pub post: PostResource,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl ImageSearchSimilarPost {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for ImageSearchSimilarPost {
    fn with_base_url(self, url: &str) -> Self {
        Self {
            post: self.post.with_base_url(url),
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
#[serde(rename_all = "camelCase")]
/// A type to represent the result from an Image search request
pub struct ImageSearchResult {
    /// A post resource that is exact byte-to-byte duplicate of the input file
    pub exact_post: Option<PostResource>,
    /// A series of post resources that aren't exact duplicate, but visually resembles
    /// the input file. Works only on images and animations, does not work for videos and
    /// Flash movies.
    pub similar_posts: Vec<ImageSearchSimilarPost>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl ImageSearchResult {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl WithBaseURL for ImageSearchResult {
    fn with_base_url(self, url: &str) -> Self {
        Self {
            exact_post: self.exact_post.with_base_url(url),
            similar_posts: self.similar_posts.with_base_url(url),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "python"),
    pyclass(get_all, module = "szurubooru_client.models")
)]
/// A type that represents posts that are before or after an existing post
pub struct AroundPostResult {
    /// A previous post, if it exists
    prev: Option<u32>,
    /// The next post, if it exists
    next: Option<u32>,
}

#[cfg(feature = "python")]
#[cfg_attr(all(feature = "python"), pymethods)]
#[doc(hidden)]
impl AroundPostResult {
    /// Generates a representative string of this resource
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{GlobalInfo, GlobalInfoConfig, SnapshotResource, TagCategoryResource};
    use chrono::Datelike;

    #[test]
    fn test_parse_global_info() {
        let cfg_str = r#"{
            "name": "integrationland",
            "userNameRegex": "^[a-zA-Z0-9_-]{1,32}$",
            "passwordRegex": "^.{5,}$",
            "tagNameRegex": "^\\S+$",
            "tagCategoryNameRegex": "^[^\\s%+#/]+$",
            "defaultUserRank": "regular",
            "enableSafety": true,
            "contactEmail": null,
            "canSendMails": false,
            "privileges": {
                "users:create:self": "anonymous",
                "users:create:any": "administrator",
                "comments:edit:own": "regular",
                "comments:list": "regular",
                "comments:view": "regular",
                "comments:score": "regular",
                "snapshots:list": "power",
                "uploads:create": "regular",
                "uploads:useDownloader": "power"
            }
        }"#;

        let global_config =
            serde_json::from_str::<GlobalInfoConfig>(cfg_str).expect("Unable to parse cfg_str");
        assert_eq!(global_config.can_send_mails, false);
        let info_str = r#"{"postCount": 0,
            "diskUsage": 0,
            "serverTime": "2024-08-09T21:41:24.123623Z",
            "config": {
                "name": "integrationland",
                "userNameRegex": "^[a-zA-Z0-9_-]{1,32}$",
                "passwordRegex": "^.{5,}$",
                "tagNameRegex": "^\\S+$",
                "tagCategoryNameRegex": "^[^\\s%+#/]+$",
                "defaultUserRank": "regular",
                "enableSafety": true,
                "contactEmail": null,
                "canSendMails": false,
                "privileges": {
                    "users:create:self": "anonymous"
                }
            },
            "featuredPost": null,
            "featuringUser": null,
            "featuringTime": null
        }"#;
        let global_info =
            serde_json::from_str::<GlobalInfo>(info_str).expect("Unable to parse info_str");
        assert_eq!(global_info.server_time.year(), 2024);
    }

    #[test]
    fn test_parse_tag_category_resource() {
        let input_str = r#"        {
            "name": "default",
            "version": 1,
            "color": "default",
            "usages": 0,
            "default": true,
            "order": 1
        }"#;
        let tag_cat = serde_json::from_str::<TagCategoryResource>(input_str)
            .expect("Unable to parse tag category string");
        assert_eq!(tag_cat.name, Some("default".to_string()));
    }

    #[test]
    fn test_parse_snapshot() {
        let input_str = r#"
        {
            "operation": "merged",
            "type": "pool",
            "id": "2",
            "user": {
                "name": "integration_user",
                "avatarUrl": "https://gravatar.com/avatar/6ab25d2babacc114ca560bff7c264d08?d=retro&s=300"
            },
            "data": [
                "pool",
                1
            ],
            "time": "2024-08-11T19:53:34.384644Z"
        }
        "#;
        serde_json::from_str::<SnapshotResource>(input_str)
            .expect("Could not parse merged snapshot resource");

        let input_str = r#"
        {
            "operation": "modified",
            "type": "pool_category",
            "id": "cat_pool_category",
            "user": {
                "name": "integration_user",
                "avatarUrl": "https://gravatar.com/avatar/6ab25d2babacc114ca560bff7c264d08?d=retro&s=300"
            },
            "data": {
                "type": "object change",
                "value": {
                    "default": {
                        "type": "primitive change",
                        "old-value": false,
                        "new-value": true
                    }
                }
            },
            "time": "2024-08-11T19:53:33.422437Z"
        }
        "#;
        serde_json::from_str::<SnapshotResource>(input_str)
            .expect("Could not parse modified snapshot resource");

        let input_str = r#"
        {
            "operation": "deleted",
            "type": "pool",
            "id": "3",
            "user": {
                "name": "integration_user",
                "avatarUrl": "https://gravatar.com/avatar/6ab25d2babacc114ca560bff7c264d08?d=retro&s=300"
            },
            "data": {
                "names": [
                    "dogs_pool"
                ],
                "category": "cat_pool_category",
                "posts": []
            },
            "time": "2024-08-11T19:53:34.006828Z"
        }
        "#;
        serde_json::from_str::<SnapshotResource>(input_str)
            .expect("Could not parse deleted snapshot resource");

        let input_str = r#"
        {
            "operation": "created",
            "type": "pool",
            "id": "1",
            "user": {
                "name": "integration_user",
                "avatarUrl": "https://gravatar.com/avatar/6ab25d2babacc114ca560bff7c264d08?d=retro&s=300"
            },
            "data": {
                "names": [
                    "cats_pool"
                ],
                "category": "cat_pool_category",
                "posts": []
            },
            "time": "2024-08-11T19:53:33.613959Z"
        }
        "#;
        serde_json::from_str::<SnapshotResource>(input_str)
            .expect("Could not parse created snapshot resource");
    }
}
