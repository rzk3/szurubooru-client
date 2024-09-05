use crate::models::*;
use crate::py::asynchronous::PythonAsyncClient;
use crate::py::PyPagedSearchResult;
use crate::tokens::QueryToken;
use chrono::{DateTime, Utc};
use pyo3::prelude::*;
use std::path::PathBuf;
use tokio::runtime::{Builder, Runtime};

#[pyclass(name = "SzurubooruSyncClient", module = "szurubooru_client")]
/// Constructor for the SzurubooruSyncClient
/// This client is completely synchronous. For the ``asyncio`` compatible version,
/// see :class:`SzurubooruAsyncClient`
///
/// :param str host: Base host URL for the Szurubooru instance. Should be the protocol, hostname and any port E.g ``http://localhost:9801``
/// :param str username: The username used to authenticate against the Szurubooru instance. Leave blank for anonymous authentication
/// :param str password: The password to use for ``Basic`` authentication. Token authentication should be preferred
/// :param str token: The token to use for ``Bearer`` authentication.
/// :param bool allow_insecure: Disable cert validation. Disables SSL authentication
///
/// :rtype: SzurubooruSyncClient
pub struct PythonSyncClient {
    client: PythonAsyncClient,
    runtime: Runtime,
}

#[pymethods]
impl PythonSyncClient {
    #[new]
    #[pyo3(signature = (host, username=None, token=None, password=None, allow_insecure=None))]
    /// This method is for creating new instances of the SzurubooruSyncClient
    pub fn new(
        host: String,
        username: Option<String>,
        token: Option<String>,
        password: Option<String>,
        allow_insecure: Option<bool>,
    ) -> PyResult<Self> {
        let runtime = Builder::new_current_thread().enable_all().build()?;
        let client = PythonAsyncClient::new(host, username, token, password, allow_insecure)?;
        Ok(Self { client, runtime })
    }

    #[pyo3(signature = (fields=None))]
    /// List the available tag categories
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    /// :return: A ``list`` of Tag Category resources
    /// :rtype: list[TagCategoryResource]
    pub fn list_tag_categories(
        &self,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<TagCategoryResource>> {
        self.runtime
            .block_on(self.client.list_tag_categories(fields))
    }

    #[pyo3(signature = (name, color=None, order=None, fields=None))]
    /// Creates a new tag category using the specified parameters.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str name: The tag category name, must match the server's ``tag_category_name_regex``
    /// :param Optional[str] color: The color name for this tag category
    /// :param Optional[str] order: The sort order for the tag category
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Tag Category resources
    /// :rtype: :class:`TagCategoryResource <szurubooru_client.models.TagCategoryResource>`
    pub fn create_tag_category(
        &self,
        name: String,
        color: Option<String>,
        order: Option<u32>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        self.runtime
            .block_on(self.client.create_tag_category(name, color, order, fields))
    }

    #[pyo3(signature = (name, version, new_name=None, color=None, order=None, fields=None))]
    /// Updates an existing tag category using the specified parameters.
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str name: The tag category name, must match the server's ``tag_category_name_regex``
    /// :param int version: The existing resource's version
    /// :param Optional[str] new_name: The new name for the tag category
    /// :param Optional[str] color: The color name for this tag category
    /// :param Optional[str] order: The sort order for the tag category
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Tag Category resource
    /// :rtype: :class:`TagCategoryResource <szurubooru_client.models.TagCategoryResource>`
    pub fn update_tag_category(
        &self,
        name: String,
        version: u32,
        new_name: Option<String>,
        color: Option<String>,
        order: Option<u32>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        self.runtime.block_on(
            self.client
                .update_tag_category(name, version, new_name, color, order, fields),
        )
    }

    #[pyo3(signature = (name, fields=None))]
    /// Fetches a tag category by name
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str name: The name of the tag category
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Tag Category resource
    /// :rtype: :class:`TagCategoryResource <szurubooru_client.models.TagCategoryResource>`
    pub fn get_tag_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        self.runtime
            .block_on(self.client.get_tag_category(name, fields))
    }

    #[pyo3(signature = (name, version))]
    /// Deletes a tag category
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str name: The tag category's name
    /// :param int version: The existing resource's version
    ///
    pub fn delete_tag_category(&self, name: String, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_tag_category(name, version))
    }

    #[pyo3(signature = (name))]
    /// Sets the default tag category for the site
    ///
    /// :param str name: The name of the category to set as default
    pub fn set_default_tag_category(&self, name: String) -> PyResult<()> {
        self.runtime
            .block_on(self.client.set_default_tag_category(name))
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// List the tags currently available on the site
    ///
    /// .. note::
    ///     This method supports :doc:`Query Tokens </tokens>`
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result limits <limits>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result offsets <offsets>`
    ///
    /// :param list[QueryToken] query: A list of query tokens used to filter the results
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    /// :param Optional[int] limit: The maximum number of resources to return
    /// :param Optional[int] offset: The number of resources to skip before returning the resulting resources
    ///
    /// :see: :class:`szurubooru_client.tokens.TagNamedToken` and :class:`~szurubooru_client.tokens.TagSortToken` for query filtering
    ///
    /// :return: A paged result of :class:`~szurubooru_client.models.TagResource`
    /// :rtype: :class:`~szurubooru_client.PagedResult`
    pub fn list_tags(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.runtime
            .block_on(self.client.list_tags(query, fields, limit, offset))
    }

    #[pyo3(signature = (names, category=None, description=None, implications=None, suggestions=None, fields=None))]
    /// Creates a new tag using specified parameters. Names, suggestions and implications must
    /// match `tag_name_regex` from server's configuration. Category must exist and is the same
    /// as the `name` field within :class:`~szurubooru_client.models.TagCategoryResource` resource.
    /// Suggestions and implications are optional. If specified implied tags or suggested tags do
    /// not exist yet, they will be automatically created. Tags created automatically have no
    /// implications, no suggestions, one name and their category is set to the first tag category
    /// found. If there are no tag categories established yet, an error will be thrown.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param names: A string or list of strings that serve as the name(s) for this tag
    /// :param str category: The name of the tag category that this tag belongs to
    /// :param list[str] implications: Tags that are automatically implied when this tag is used
    /// :param list[str] suggestions: Tags that should be suggested when this tag is used
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Tag resource
    /// :rtype: :class:`TagResource <szurubooru_client.models.TagResource>`
    pub fn create_tag(
        &self,
        names: Py<PyAny>,
        category: Option<String>,
        description: Option<String>,
        implications: Option<Vec<String>>,
        suggestions: Option<Vec<String>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagResource> {
        self.runtime.block_on(self.client.create_tag(
            names,
            category,
            description,
            implications,
            suggestions,
            fields,
        ))
    }

    #[pyo3(signature = (name, version, names=None, category=None, description=None,
        implications=None, suggestions=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Updates an existing tag using specified parameters. Names, suggestions and implications must
    /// match `tag_name_regex` from server's configuration. Category must exist and is the same
    /// as the `name` field within :class:`~szurubooru_client.models.TagCategoryResource` resource.
    /// Suggestions and implications are optional. If specified implied tags or suggested tags do
    /// not exist yet, they will be automatically created. Tags created automatically have no
    /// implications, no suggestions, one name and their category is set to the first tag category
    /// found. If there are no tag categories established yet, an error will be thrown.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param name: The name of the existing tags
    /// :param int version: The existing resource's version
    /// :param Optional[list|str] names: A string or list of strings that the tag should be known as
    /// :param str category: The name of the tag category that this tag belongs to
    /// :param list[str] implications: Tags that are automatically implied when this tag is used
    /// :param list[str] suggestions: Tags that should be suggested when this tag is used
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Tag resource
    /// :rtype: :class:`TagResource <szurubooru_client.models.TagResource>`
    pub fn update_tag(
        &self,
        name: String,
        version: u32,
        names: Option<Py<PyAny>>,
        category: Option<String>,
        description: Option<String>,
        implications: Option<Vec<String>>,
        suggestions: Option<Vec<String>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagResource> {
        self.runtime.block_on(self.client.update_tag(
            name,
            version,
            names,
            category,
            description,
            implications,
            suggestions,
            fields,
        ))
    }

    #[pyo3(signature = (name, fields=None))]
    /// Fetches an existing tag
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str name: The name of the tag to fetch
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Tag resource
    /// :rtype: :class:`~szurubooru_client.models.TagResource`
    pub fn get_tag(&self, name: String, fields: Option<Vec<String>>) -> PyResult<TagResource> {
        self.runtime.block_on(self.client.get_tag(name, fields))
    }

    #[pyo3(signature = (name, version))]
    /// Deletes an existing tag
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str name: The name of the tag to delete
    /// :param int version: The existing resource's version
    pub fn delete_tag(&self, name: String, version: u32) -> PyResult<()> {
        self.runtime.block_on(self.client.delete_tag(name, version))
    }

    #[pyo3(signature = (remove_tag, remove_tag_version, merge_to_tag, merge_to_version, fields=None))]
    /// Removes source tag and merges all of its usages, suggestions and implications to the
    /// target tag. Other tag properties such as category and aliases do not get transferred
    /// and are discarded.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires two resource versions. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str remove_tag: The name of the tag to be removed
    /// :param int remove_tag_version: The current version of the tag to be removed
    /// :param str merge_to_tag: The name of the tag to be merged *to*
    /// :param int merge_to_version: The current version of the tag to merge *to*
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Tag resource
    /// :rtype: :class:`~szurubooru_client.models.TagResource`
    pub fn merge_tags(
        &self,
        remove_tag: String,
        remove_tag_version: u32,
        merge_to_tag: String,
        merge_to_version: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagResource> {
        self.runtime.block_on(self.client.merge_tags(
            remove_tag,
            remove_tag_version,
            merge_to_tag,
            merge_to_version,
            fields,
        ))
    }

    #[pyo3(signature = (name))]
    /// Lists siblings of given tag, e.g. tags that were used in the same posts as the given tag.
    /// The ``occurrences`` field signifies how many times a given
    /// sibling appears with given tag. Results are sorted by occurrences count and the list is
    /// truncated to the first 50 elements.
    ///
    /// :param str name: The name of the tag to fetch siblings for
    ///
    /// :return: A list of Tag siblings
    /// :rtype: list[TagSibling]
    pub fn get_tag_siblings(&self, name: String) -> PyResult<Vec<TagSibling>> {
        self.runtime.block_on(self.client.get_tag_siblings(name))
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// Lists the posts currently available on the site
    ///
    /// .. note::
    ///     This method supports :doc:`Query Tokens </tokens>`
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result limits <limits>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result offsets <offsets>`
    ///
    /// :param Optional[list[QueryToken]] query: A list of query tokens used to filter the results
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    /// :param Optional[int] limit: The maximum number of resources to return
    /// :param Optional[int] offset: The number of results to skip before returning the result
    ///
    /// :see: :class:`szurubooru_client.tokens.PostNamedToken`, :class:`~szurubooru_client.tokens.PostSortToken`, and :class:`~szurubooru_client.tokens.PostSpecialToken` for query filtering
    ///
    /// :return: A :class:`~szurubooru_client.PagedResult` of Post resources
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn list_posts(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.runtime
            .block_on(self.client.list_posts(query, fields, limit, offset))
    }

    #[pyo3(signature = (url=None, upload_token=None, file_path=None, thumbnail_path=None, tags=None, safety=None, source=None,
            relations=None, notes=None, flags=None, anonymous=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Creates a new post using one of three image sources: URL, upload token or file path.
    ///
    /// * URL: The server will download the given Image URL as the post's content
    /// * Upload token: The token returned by using :func:`~szurubooru_client.SzurubooruSyncClient.upload_temporary_file`
    /// * File path: The ``pathlib.Path`` or ``str`` path to the file to be uploaded from the local filesystem
    ///
    /// .. warning::
    ///     The ``safety`` argument is *required*
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param Optional[str] url: The URL of the image to use for the post's content
    /// :param Optional[str] upload_token: The token returned by the temporary upload method
    /// :param Optional[str|pathlib.Path] file_path: The local file path to upload
    /// :param Optional[str|Path] thumbnail_path: The local file path to the thumbnail for the post
    /// :param Optional[list[str]] tags: The list of tag names to use for the post
    /// :param PostSafety safety: The safety level of the post
    /// :param Optional[list[int]] relations: A list of related post IDs
    /// :param Optional[list[NoteResource]] notes: A list of :class:`~szurubooru_client.models.NoteResource` for the post
    /// :param Optional[list[str]] flags: A list of flags to apply to the post
    /// :param Optional[bool] anonymous: Whether to create the post anonymously
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Post resource
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn create_post(
        &self,
        url: Option<String>,
        upload_token: Option<String>,
        file_path: Option<PathBuf>,
        thumbnail_path: Option<PathBuf>,
        tags: Option<Vec<String>>,
        safety: Option<PostSafety>,
        source: Option<String>,
        relations: Option<Vec<u32>>,
        notes: Option<Vec<NoteResource>>,
        flags: Option<Vec<String>>,
        anonymous: Option<bool>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime.block_on(self.client.create_post(
            url,
            upload_token,
            file_path,
            thumbnail_path,
            tags,
            safety,
            source,
            relations,
            notes,
            flags,
            anonymous,
            fields,
        ))
    }

    #[pyo3(signature = (post_id, post_version, url=None, token=None, file_path=None,
        thumbnail_path=None, tags=None, safety=None, source=None, relations=None, notes=None,
        flags=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Updates an existing post
    ///
    /// The post's content can be replaced using one of three methods:
    /// * URL: The server will download the given Image URL as the post's content
    /// * Upload token: The token returned by using :func:`~szurubooru_client.SzurubooruSyncClient.upload_temporary_file`
    /// * File path: The ``pathlib.Path`` or string path to the file to be uploaded.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param int post_id: The ID of the post to update
    /// :param int version: The existing resource's version
    /// :param Optional[str] url: The URL of the image to use for the post's content
    /// :param Optional[str] upload_token: The token returned by the temporary upload method
    /// :param Optional[str|Path] file_path: The local file path to upload
    /// :param Optional[str|Path] thumbnail_path: The local file path to the thumbnail for the post
    /// :param Optional[list[str]] tags: The list of tag names to use for the post
    /// :param Optional[PostSafety] safety: The safety level of the post
    /// :param Optional[list[int]] relations: A list of related post IDs
    /// :param Optional[list[NoteResource]] notes: A list of :class:`~szurubooru_client.models.NoteResource` for the post
    /// :param Optional[list[str]] flags: A list of flags to apply to the post
    /// :param Optional[bool] anonymous: Whether to create the post anonymously
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Post resource
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn update_post(
        &self,
        post_id: u32,
        post_version: u32,
        url: Option<String>,
        token: Option<String>,
        file_path: Option<PathBuf>,
        thumbnail_path: Option<PathBuf>,
        tags: Option<Vec<String>>,
        safety: Option<PostSafety>,
        source: Option<String>,
        relations: Option<Vec<u32>>,
        notes: Option<Vec<NoteResource>>,
        flags: Option<Vec<String>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime.block_on(self.client.update_post(
            post_id,
            post_version,
            url,
            token,
            file_path,
            thumbnail_path,
            tags,
            safety,
            source,
            relations,
            notes,
            flags,
            fields,
        ))
    }

    #[pyo3(signature = (post_id))]
    /// Downloads the given post's image as a byte array
    ///
    /// :param int post_id: The ID of the post to fetch
    ///
    /// :return: A byte array of the given post's content
    /// :rtype: list[byte]
    pub fn get_image_bytes(&self, post_id: u32) -> PyResult<Vec<u8>> {
        self.runtime.block_on(self.client.get_image_bytes(post_id))
    }

    #[pyo3(signature = (post_id, file_path))]
    /// Downloads the given post's image to a path on the filesystem
    ///
    /// :param int post_id: The ID of the post to fetch
    /// :param Path|str file_path: The path to download the image to
    pub fn download_image_to_path(&self, post_id: u32, file_path: PathBuf) -> PyResult<()> {
        self.runtime
            .block_on(self.client.download_image_to_path(post_id, file_path))
    }

    #[pyo3(signature = (post_id))]
    /// Downloads the given post's thumbnail as a byte array
    ///
    /// :param int post_id: The ID of the post to fetch
    ///
    /// :return: A byte array of the given post's content
    /// :rtype: list[byte]
    pub fn get_thumbnail_bytes(&self, post_id: u32) -> PyResult<Vec<u8>> {
        self.runtime
            .block_on(self.client.get_thumbnail_bytes(post_id))
    }

    #[pyo3(signature = (post_id, file_path))]
    /// Downloads the given post's thumbnail to a path on the filesystem
    ///
    /// :param int post_id: The ID of the post to fetch
    /// :param Path|str file_path: The path to download the thumbnail to
    pub fn download_thumbnail_to_path(&self, post_id: u32, file_path: PathBuf) -> PyResult<()> {
        self.runtime
            .block_on(self.client.download_thumbnail_to_path(post_id, file_path))
    }

    #[pyo3(signature = (image_path))]
    /// Reverse image searches for an image from the filesystem. Returns
    /// a list of visually similar images
    ///
    /// :param Path|str image_path: The path to the image to search for
    ///
    /// :return: An object containing the IDs of similar posts
    /// :rtype: :class:`~szurubooru_client.models.ImageSearchResult`
    pub fn reverse_image_search(&self, image_path: PathBuf) -> PyResult<ImageSearchResult> {
        self.runtime
            .block_on(self.client.reverse_image_search(image_path))
    }

    #[pyo3(signature = (image_path))]
    /// Searches for an *exact* image match of an image from the filesystem
    ///
    /// :param Path|str image_path: The path to the image to search for
    ///
    /// :return: A Post Resource or None if the image doesn't exist
    /// :rtype: None|:class:`~szurubooru_client.models.PostResource`
    pub fn post_for_image(&self, image_path: PathBuf) -> PyResult<Option<PostResource>> {
        self.runtime
            .block_on(self.client.post_for_image(image_path))
    }

    #[pyo3(signature = (post_id, fields=None))]
    /// Fetches an individual post by its post ID
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param int post_id: The ID of the post to fetch
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A Post resource
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn get_post(&self, post_id: u32, fields: Option<Vec<String>>) -> PyResult<PostResource> {
        self.runtime.block_on(self.client.get_post(post_id, fields))
    }

    #[pyo3(signature = (post_id))]
    /// Fetches posts from *around* the given post ID. That means the post before and after,
    /// if they exist.
    ///
    /// :param int post_id: The ID of the post to fetch
    ///
    /// :return: A resource containing the IDs of the next and previous IDs
    /// :rtype: :class:`~szurubooru_client.models.AroundPostResult`
    pub fn get_around_post(&self, post_id: u32) -> PyResult<AroundPostResult> {
        self.runtime.block_on(self.client.get_around_post(post_id))
    }

    #[pyo3(signature = (post_id, version))]
    /// Deletes a post by its ID
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param int post_id: The ID of the post to delete
    /// :param int version: The existing resource's version
    pub fn delete_post(&self, post_id: u32, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_post(post_id, version))
    }

    #[pyo3(signature = (remove_post, remove_post_version, merge_to_post,
        merge_to_version, replace_post_content=false, fields=None))]
    /// Removes source post and merges all of its tags, relations, scores, favorites and comments to
    /// the target post. If ``replace_content`` is set to ``true``, content of the target post
    /// is replaced using the content of the source post; otherwise it remains unchanged. Source
    /// post properties such as its safety, source, whether to loop the video and other scalar
    /// values do not get transferred and are discarded.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires two resource versions. See :ref:`Resource Versioning <rver>`
    ///
    /// :param int remove_post: The ID of the source post
    /// :param int remove_post_version: The current version of the source post
    /// :param int merge_to_post: The ID of the destination post
    /// :param int merge_to_version: The current version of the destination post
    /// :param bool replace_post_content: Whether to replace the destination post's content with the content from the source post
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A post resource
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn merge_post(
        &self,
        remove_post: u32,
        remove_post_version: u32,
        merge_to_post: u32,
        merge_to_version: u32,
        replace_post_content: bool,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime.block_on(self.client.merge_post(
            remove_post,
            remove_post_version,
            merge_to_post,
            merge_to_version,
            replace_post_content,
            fields,
        ))
    }

    #[pyo3(signature = (post_id, rating, fields=None))]
    /// Updates score of authenticated user for given post. Valid scores are -1, 0 and 1.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param int post_id: The ID of the post to rate
    /// :param int rating: The rating to give the post. Must be -1, 0 or 1.
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A post resource
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn rate_post(
        &self,
        post_id: u32,
        rating: i8,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime
            .block_on(self.client.rate_post(post_id, rating, fields))
    }

    #[pyo3(signature = (post_id, fields=None))]
    /// Marks the post as favorite for the current user.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param int post_id: The ID of the post to rate
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A post resource
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn favorite_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime
            .block_on(self.client.favorite_post(post_id, fields))
    }

    #[pyo3(signature = (post_id, fields=None))]
    /// Unmarks the post as favorite for the current user.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param int post_id: The ID of the post to rate
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A post resource
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn unfavorite_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime
            .block_on(self.client.unfavorite_post(post_id, fields))
    }

    #[pyo3(signature = (fields=None))]
    /// Retrieves the post that is currently featured on the main page. If no post is
    /// featured, the returned value is ``None``. Note that this method exists mostly for compatibility
    /// with setting featured post - most of the time, you'd want to use query global info which
    /// contains more information.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A post resource or ``None``
    /// :rtype: Optional[:class:`~szurubooru_client.models.PostResource`]
    pub fn get_featured_post(&self, fields: Option<Vec<String>>) -> PyResult<Option<PostResource>> {
        self.runtime.block_on(self.client.get_featured_post(fields))
    }

    #[pyo3(signature = (post_id, fields=None))]
    /// Features a post on the main page
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param int post_id: The ID of the post to feature
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A post resource
    /// :rtype: :class:`~szurubooru_client.models.PostResource`
    pub fn set_featured_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime
            .block_on(self.client.set_featured_post(post_id, fields))
    }

    #[pyo3(signature = (fields=None))]
    /// Lists all pool categories
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A list of Pool Category resources
    /// :rtype: list[:class:`~szurubooru_client.models.PoolCategoryResource`]
    pub fn list_pool_categories(
        &self,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<PoolCategoryResource>> {
        self.runtime
            .block_on(self.client.list_pool_categories(fields))
    }

    #[pyo3(signature = (name, color=None, fields=None))]
    /// Creates a new pool category using specified parameters. Name must match
    /// ``pool_category_name_regex`` from server's configuration. First category created becomes
    /// the default category.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str name: The name of the pool category to create
    /// :param str color: The color to associate with the pool category
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A pool category resource
    /// :rtype: :class:`~szurubooru_client.models.PoolCategoryResource`
    pub fn create_pool_category(
        &self,
        name: String,
        color: Option<String>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        self.runtime
            .block_on(self.client.create_pool_category(name, color, fields))
    }

    #[pyo3(signature = (name, version, new_name=None, color=None, fields=None))]
    /// Updates an existing tag category using specified parameters. Name must match
    /// `tag_category_name_regex` from server's configuration.
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str name: The name of the pool category to modify
    /// :param int version: The existing resource's version
    /// :param Optional[str] new_name: The new name for the pool category
    /// :param Optional[str] color: The new color for the pool category
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: An updated pool category resource
    /// :rtype: :class:`~szurubooru_client.models.PoolCategoryResource`
    pub fn update_pool_category(
        &self,
        name: String,
        version: u32,
        new_name: Option<String>,
        color: Option<String>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        self.runtime.block_on(
            self.client
                .update_pool_category(name, version, new_name, color, fields),
        )
    }

    #[pyo3(signature = (name, fields=None))]
    /// Fetches an existing pool category
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str name: The name of the pool category to fetch
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A pool category resource
    /// :rtype: :class:`~szurubooru_client.models.PoolCategoryResource`
    pub fn get_pool_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        self.runtime
            .block_on(self.client.get_pool_category(name, fields))
    }

    #[pyo3(signature = (name, version))]
    /// Deletes existing pool category. The pool category to be deleted must have no usages.
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str name: The name of the pool category to delete
    /// :param int version: The existing resource's version
    pub fn delete_pool_category(&self, name: String, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_pool_category(name, version))
    }

    #[pyo3(signature = (name, fields=None))]
    /// Sets given pool category as default. All new pools created manually or automatically will
    /// have this category.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str name: The name of the pool category to be set as default
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A pool category resource
    /// :rtype: :class:`~szurubooru_client.models.PoolCategoryResource`
    pub fn set_default_pool_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        self.runtime
            .block_on(self.client.set_default_pool_category(name, fields))
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// List the post pools currently available on the site
    ///
    /// .. note::
    ///     This method supports :doc:`Query Tokens </tokens>`
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result limits <limits>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result offsets <offsets>`
    ///
    /// :param list[QueryToken] query: A list of query tokens used to filter the results
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    /// :param Optional[int] limit: The maximum number of resources to return
    /// :param Optional[int] offset: The number of resources to skip before returning the resulting resources
    ///
    /// :see: :class:`szurubooru_client.tokens.PoolNamedToken` and :class:`~szurubooru_client.tokens.PoolSortToken` for query filtering
    ///
    /// :return: A paged result of :class:`~szurubooru_client.models.PoolResource`
    /// :rtype: :class:`~szurubooru_client.PagedResult`
    pub fn list_pools(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.runtime
            .block_on(self.client.list_pools(query, fields, limit, offset))
    }

    #[pyo3(signature = (names, category=None, description=None, posts=None, fields=None))]
    /// Creates a new pool using specified parameters. Names, suggestions and implications must
    /// match `pool_name_regex` from server's configuration. ``posts`` is an optional list of
    /// post IDs to add to the pool. If the specified posts do not exist, an error will be thrown
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param list[str]|str names: The name or names for the new pool
    /// :param Optional[str] category: The pool category for this pool
    /// :param Optional[str] description: The description for this pool
    /// :param Optional[list[int]] posts: The posts that will be part of this pool
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A pool resource
    /// :rtype: :class:`~szurubooru_client.models.PoolResource`
    pub fn create_pool(
        &self,
        names: Py<PyAny>,
        category: Option<String>,
        description: Option<String>,
        posts: Option<Vec<u32>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        self.runtime.block_on(
            self.client
                .create_pool(names, category, description, posts, fields),
        )
    }

    #[pyo3(signature = (pool_id, version, new_names=None, category=None, description=None,
        posts=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Updates an existing pool using specified parameters.
    /// ``new_names``, if given, must match ``pool_name_regex`` from server's configuration.
    /// ``category``, if given, must exist.
    /// ``posts`` is an optional list of integer post IDs. If the specified posts do not exist yet,
    /// an error will be thrown. The full list of post IDs must be provided if they are being
    /// updated, and the previous list of posts will be replaced with the new one.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param int pool_id: The ID of the pool to update
    /// :param int version: The existing resource's version
    /// :param Optional[list[str]] new_names: The new name(s) for the pool
    /// :param Optional[str] category: The new pool category for the pool
    /// :param Optional[str] description: The new description for the pool
    /// :param Optional[list[int]] posts: The posts that belong to this pool
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A pool category resource
    /// :rtype: :class:`~szurubooru_client.models.PoolResource`
    pub fn update_pool(
        &self,
        pool_id: u32,
        version: u32,
        new_names: Option<Vec<String>>,
        category: Option<String>,
        description: Option<String>,
        posts: Option<Vec<u32>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        self.runtime.block_on(self.client.update_pool(
            pool_id,
            version,
            new_names,
            category,
            description,
            posts,
            fields,
        ))
    }

    #[pyo3(signature = (pool_id, fields=None))]
    /// Retrieves information about an existing pool
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param int pool_id: The ID of the pool to fetch
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A pool resource
    /// :rtype: :class:`~szurubooru_client.models.PoolResource`
    pub fn get_pool(&self, pool_id: u32, fields: Option<Vec<String>>) -> PyResult<PoolResource> {
        self.runtime.block_on(self.client.get_pool(pool_id, fields))
    }

    #[pyo3(signature = (pool_id, version))]
    /// Deletes existing pool. All posts in the pool will only have their relation to the pool
    /// removed.
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param int pool_id: The ID of the pool to delete
    /// :param int version: The existing resource's version
    pub fn delete_pool(&self, pool_id: u32, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_pool(pool_id, version))
    }

    #[pyo3(signature = (remove_pool, remove_pool_version, merge_to_pool, merge_to_version, fields=None))]
    /// Removes source pool and merges all of its posts with the target pool. Other pool properties
    /// such as category and aliases do not get transferred and are discarded.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires two resource versions. See :ref:`Resource Versioning <rver>`
    ///
    /// :param int remove_pool: The ID of the source pool
    /// :param int remove_pool_version: The current version of the source pool
    /// :param int merge_to_pool: The ID of the destination pool
    /// :param int merge_to_version: The current version of the destination pool
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A pool resource
    /// :rtype: :class:`~szurubooru_client.models.PoolResource`
    pub fn merge_pools(
        &self,
        remove_pool: u32,
        remove_pool_version: u32,
        merge_to_pool: u32,
        merge_to_version: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        self.runtime.block_on(self.client.merge_pools(
            remove_pool,
            remove_pool_version,
            merge_to_pool,
            merge_to_version,
            fields,
        ))
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// List the comments currently available on the site.
    ///
    ///
    /// .. note::
    ///     This method supports :doc:`Query Tokens </tokens>`
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result limits <limits>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result offsets <offsets>`
    ///
    /// :param list[QueryToken] query: A list of query tokens used to filter the results
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    /// :param Optional[int] limit: The maximum number of resources to return
    /// :param Optional[int] offset: The number of resources to skip before returning the resulting resources
    ///
    /// :see: :class:`~szurubooru_client.tokens.CommentNamedToken` and `~szurubooru_client.tokens.CommentSortToken` for query filtering
    ///
    /// :return: A paged result of :class:`~szurubooru_client.models.CommentResource`
    /// :rtype: :class:`~szurubooru_client.PagedResult`
    pub fn list_comments(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.runtime
            .block_on(self.client.list_comments(query, fields, limit, offset))
    }

    #[pyo3(signature = (text, post_id, fields=None))]
    /// Creates a new comment under a given post
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str text: The text of the comment
    /// :param int post_id: The ID of the post to create the comment on
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A comment resource
    /// :rtype: :class:`~szurubooru_client.models.CommentResource`
    pub fn create_comment(
        &self,
        text: String,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        self.runtime
            .block_on(self.client.create_comment(text, post_id, fields))
    }

    #[pyo3(signature = (comment_id, version, text, fields=None))]
    /// Updates an existing comment with new text
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param int comment_id: The ID of the comment to update
    /// :param int version: The existing resource's version
    /// :param str text: The new text for the comment
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A comment resource
    /// :rtype: :class:`~szurubooru_client.models.CommentResource`
    pub fn update_comment(
        &self,
        comment_id: u32,
        version: u32,
        text: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        self.runtime.block_on(
            self.client
                .update_comment(comment_id, version, text, fields),
        )
    }

    #[pyo3(signature = (comment_id, fields=None))]
    /// Fetches an existing comment
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param int comment_id: The ID of the comment to fetch
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A comment resource
    /// :rtype: :class:`~szurubooru_client.models.CommentResource`
    pub fn get_comment(
        &self,
        comment_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        self.runtime
            .block_on(self.client.get_comment(comment_id, fields))
    }

    #[pyo3(signature = (comment_id, version))]
    /// Deletes an existing comment
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param int comment_id: The ID of the comment to delete
    /// :param int version: The existing resource's version
    pub fn delete_comment(&self, comment_id: u32, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_comment(comment_id, version))
    }

    #[pyo3(signature = (comment_id, rating, fields=None))]
    /// Updates score of authenticated user for given comment. Valid scores are -1, 0 and 1.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param int comment_id: The ID of the comment to rate
    /// :param int rating: The rating to give the comment. Must be -1, 0, or 1
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A comment resource
    /// :rtype: :class:`~szurubooru_client.models.CommentResource`
    pub fn rate_comment(
        &self,
        comment_id: u32,
        rating: i8,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        self.runtime
            .block_on(self.client.rate_comment(comment_id, rating, fields))
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// List the users currently registered on the site
    ///
    /// .. note::
    ///     This method supports :doc:`Query Tokens </tokens>`
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result limits <limits>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result offsets <offsets>`
    ///
    /// :param list[QueryToken] query: A list of query tokens used to filter the results
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    /// :param Optional[int] limit: The maximum number of resources to return
    /// :param Optional[int] offset: The number of resources to skip before returning the resulting resources
    ///
    /// :see: :class:`szurubooru_client.tokens.UserNamedToken` and :class:`~szurubooru_client.tokens.UserSortToken` for query filtering
    ///
    /// :return: A paged result of :class:`~szurubooru_client.models.UserResource`
    /// :rtype: :class:`~szurubooru_client.PagedResult`
    pub fn list_users(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.runtime
            .block_on(self.client.list_users(query, fields, limit, offset))
    }

    #[pyo3(signature = (name, password, rank=None, avatar_path=None, fields=None))]
    /// Creates a new user using specified parameters. Names and passwords must match
    /// ``user_name_regex`` and ``password_regex`` from server's configuration, respectively.
    /// Email address, rank and avatar fields are optional. Avatar style can be either
    /// ``Gravatar`` or ``Manual`` from the :class`~szurubooru_client.models.UserAvatarStyle` enum.
    /// ``Manual`` avatar style requires client to pass also the ``avatar_path`` argument.
    /// If the rank is empty and the user happens to be the first user ever created,
    /// become an administrator, whereas subsequent users will be given the rank indicated by
    /// ``default_rank`` in the server's configuration.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str name: The new user's username
    /// :param str password: The new user's password
    /// :param Optional[UserRank] rank: The rank to give the new user
    /// :param Optional[str] avatar_path: The local file path to the user's avatar image
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A user resource
    /// :rtype: :class:`~szurubooru_client.models.UserResource`
    pub fn create_user(
        &self,
        name: String,
        password: String,
        rank: Option<UserRank>,
        avatar_path: Option<PathBuf>,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserResource> {
        self.runtime.block_on(
            self.client
                .create_user(name, password, rank, avatar_path, fields),
        )
    }

    #[pyo3(signature = (name, version, new_name=None, password=None, rank=None, avatar_path=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Updates an existing user using specified parameters. Names and passwords must match
    /// ``user_name_regex`` and ``password_regex`` from server's configuration, respectively.
    /// Email address, rank and avatar fields are optional. Avatar style can be either
    /// ``Gravatar`` or ``Manual`` from the :class`~szurubooru_client.models.UserAvatarStyle` enum.
    /// ``Manual`` avatar style requires client to pass also the ``avatar_path`` argument.
    /// If the rank is empty and the user happens to be the first user ever created,
    /// become an administrator, whereas subsequent users will be given the rank indicated by
    /// ``default_rank`` in the server's configuration.
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str name: The existing user's username
    /// :param int version: The existing resource's version
    /// :param Optional[str] new_name: The user new username
    /// :param Optional[str] password: The existing user's password
    /// :param Optional[UserRank] rank: The rank to give the existing user
    /// :param Optional[str] avatar_path: The local file path to the user's new avatar image
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A user resource
    /// :rtype: :class:`~szurubooru_client.models.UserResource`
    pub fn update_user(
        &self,
        name: String,
        version: u32,
        new_name: Option<String>,
        password: Option<String>,
        rank: Option<UserRank>,
        avatar_path: Option<PathBuf>,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserResource> {
        self.runtime.block_on(self.client.update_user(
            name,
            version,
            new_name,
            password,
            rank,
            avatar_path,
            fields,
        ))
    }

    #[pyo3(signature = (user_name, fields=None))]
    /// Retrieves information about an existing user
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str user_name: The username of the user to fetch
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A user resource
    /// :rtype: :class:`~szurubooru_client.models.UserResource`
    pub fn get_user(
        &self,
        user_name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserResource> {
        self.runtime
            .block_on(self.client.get_user(user_name, fields))
    }

    #[pyo3(signature = (user_name, version))]
    /// Deletes an existing user
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str user_name: The username of the user to delete
    /// :param int version: The existing resource's version
    pub fn delete_user(&self, user_name: String, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_user(user_name, version))
    }

    #[pyo3(signature = (user_name, fields=None))]
    /// Fetches a list of the given user's auth tokens
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str user_name: The username of the user to fetch the auth tokens for
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A user auth token resource
    /// :rtype: :class:`~szurubooru_client.models.UserAuthTokenResource`
    pub fn list_user_tokens(
        &self,
        user_name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<UserAuthTokenResource>> {
        self.runtime
            .block_on(self.client.list_user_tokens(user_name, fields))
    }

    #[pyo3(signature = (user_name, note=None, enabled=None, expiration_time=None, fields=None))]
    /// Creates an auth token for the given user
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// :param str user_name: The username of the user to create an auth token for
    /// :param Optional[str] note: A text note to include with the token
    /// :param Optional[bool] enabled: Whether the token is enabled or not
    /// :param Optional[DateTime] expiration_time: The ``DateTime`` specifying when the token should expire
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A user auth token resource
    /// :rtype: :class:`~szurubooru_client.models.UserAuthTokenResource`
    pub fn create_user_token(
        &self,
        user_name: String,
        note: Option<String>,
        enabled: Option<bool>,
        expiration_time: Option<DateTime<Utc>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserAuthTokenResource> {
        self.runtime.block_on(self.client.create_user_token(
            user_name,
            note,
            enabled,
            expiration_time,
            fields,
        ))
    }

    #[pyo3(signature = (user_name, token, version, enabled=None, note=None, expiration_time=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Update a user's existing auth token
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str user_name: The user's username
    /// :param str token: The token to update
    /// :param int version: The existing resource's version
    /// :param Optional[str] note: A text note to include with the token
    /// :param Optional[bool] enabled: Whether the token is enabled or not
    /// :param Optional[DateTime] expiration_time: The ``DateTime`` specifying when the token should expire
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    ///
    /// :return: A user auth token resource
    /// :rtype: :class:`~szurubooru_client.models.UserAuthTokenResource`
    pub fn update_user_token(
        &self,
        user_name: String,
        token: String,
        version: u32,
        enabled: Option<bool>,
        note: Option<String>,
        expiration_time: Option<DateTime<Utc>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserAuthTokenResource> {
        self.runtime.block_on(self.client.update_user_token(
            user_name,
            token,
            version,
            enabled,
            note,
            expiration_time,
            fields,
        ))
    }

    #[pyo3(signature = (user_name, token, version))]
    /// Deletes an existing user auth token
    ///
    /// .. note::
    ///     This method requires a resource version. See :ref:`Resource Versioning <rver>`
    ///
    /// :param str user_name: The user's username
    /// :param str token: The token value
    /// :param int version: The existing resource's version
    pub fn delete_user_token(
        &self,
        user_name: String,
        token: String,
        version: u32,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_user_token(user_name, token, version))
    }

    #[pyo3(signature = (email_or_name))]
    /// Start a password reset request
    ///
    /// :param str email_or_name: The email or username of the user to request the reset for
    pub fn password_reset_request(&self, email_or_name: String) -> PyResult<()> {
        self.runtime
            .block_on(self.client.password_reset_request(email_or_name))
    }

    #[pyo3(signature = (email_or_name, reset_token))]
    /// Confirm a password reset request
    ///
    /// :param str email_or_name: The email or username of the user to confirm the reset request
    /// :param str reset_token: The token sent to the user's email
    ///
    /// :return: A new temporary password
    /// :rtype: str
    pub fn password_reset_confirm(
        &self,
        email_or_name: String,
        reset_token: String,
    ) -> PyResult<String> {
        self.runtime.block_on(
            self.client
                .password_reset_confirm(email_or_name, reset_token),
        )
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// List the snapshots currently available on the site
    ///
    /// .. note::
    ///     This method supports :doc:`Query Tokens </tokens>`
    ///
    /// .. note::
    ///     This method supports :doc:`Field selection </fields>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result limits <limits>`
    ///
    /// .. note::
    ///     This method supports :ref:`Result offsets <offsets>`
    ///
    /// :param list[QueryToken] query: A list of query tokens used to filter the results
    /// :param Optional[list[str]] fields: A list of fields to select for the returned object
    /// :param Optional[int] limit: The maximum number of resources to return
    /// :param Optional[int] offset: The number of resources to skip before returning the resulting resources
    ///
    /// :see: :class:`szurubooru_client.tokens.SnapshotNamedToken` for query filtering
    ///
    /// :return: A paged result of :class:`~szurubooru_client.models.SnapshotResource`
    /// :rtype: :class:`~szurubooru_client.PagedResult`
    pub fn list_snapshots(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.runtime
            .block_on(self.client.list_snapshots(query, fields, limit, offset))
    }

    /// Retrieves simple statistics. ``featured_post`` is ``None`` if there is no featured post yet.
    /// ``server_time`` is pretty much the same as the Date HTTP
    /// field, only formatted in a manner consistent with other dates. Values in config key are
    /// taken directly from the server config, with the exception of privilege array keys being
    /// converted to lower camel case to match the API convention.
    ///
    /// :return: A Global Info object
    /// :rtype: :class:`~szurubooru_client.models.GlobalInfo`
    pub fn global_info(&self) -> PyResult<GlobalInfo> {
        self.runtime.block_on(self.client.global_info())
    }

    /// Puts a file from a given file path in temporary storage and assigns it a token that can be
    /// used in other requests.
    /// The files uploaded that way are deleted after a short while so clients shouldn't use it
    /// as a free upload service.
    ///
    /// :param Path|str file_path: The path to the file to upload from the local filesystem
    ///
    /// :return: A token that represents the uploaded image
    /// :rtype: str
    pub fn upload_temporary_file(&self, file_path: PathBuf) -> PyResult<String> {
        self.runtime
            .block_on(self.client.upload_temporary_file(file_path))
    }
}
