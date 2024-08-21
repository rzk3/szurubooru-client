use crate::models::*;
use crate::py::asynchronous::PythonAsyncClient;
use crate::py::PyPagedSearchResult;
use crate::tokens::QueryToken;
use chrono::{DateTime, Utc};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};
use std::path::{Path, PathBuf};
use tokio::runtime::{Builder, Runtime};

#[pyclass(name = "SzurubooruSyncClient")]
/// Constructor for the SzurubooruSyncClient
/// This client is completely synchronous. For the `asyncio` compatible version,
/// see [szurubooru_client.PythonAsyncClient](SzurubooruAsyncClient)
///
/// ## Arguments
/// * `host`: Base host URL for the Szurubooru instance. Should be the protocol, hostname and any port
///     E.g `http://localhost:9801`
/// * `username`: The username used to authenticate against the Szurubooru instance. Leave blank for
///     anonymous authentication
/// * `password`: The password to use for `Basic` authentication. Token authentication should
///     be preferred
/// * `token`: The token to use for `Bearer` authentication.
/// * `allow_insecure`: Disable cert validation. Disables SSL authentication
///
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
    pub fn list_tag_categories(
        &self,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<TagCategoryResource>> {
        self.runtime
            .block_on(self.client.list_tag_categories(fields))
    }

    #[pyo3(signature = (name, color=None, order=None, fields=None))]
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

    #[pyo3(signature = (name, version, color=None, order=None, fields=None))]
    pub fn update_tag_category(
        &self,
        name: String,
        version: u32,
        color: Option<String>,
        order: Option<u32>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        self.runtime.block_on(
            self.client
                .update_tag_category(name, version, color, order, fields),
        )
    }

    #[pyo3(signature = (name, fields=None))]
    pub fn get_tag_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        self.runtime
            .block_on(self.client.get_tag_category(name, fields))
    }

    pub fn delete_tag_category(&self, name: String, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_tag_category(name, version))
    }

    pub fn set_default_tag_category(&self, name: String) -> PyResult<()> {
        self.runtime
            .block_on(self.client.set_default_tag_category(name))
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
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
    pub fn create_tag(
        &self,
        //names: Vec<String>,
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
    pub fn update_tag(
        &self,
        name: String,
        version: u32,
        names: Option<Vec<String>>,
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
    pub fn get_tag(&self, name: String, fields: Option<Vec<String>>) -> PyResult<TagResource> {
        self.runtime.block_on(self.client.get_tag(name, fields))
    }

    pub fn delete_tag(&self, name: String, version: u32) -> PyResult<()> {
        self.runtime.block_on(self.client.delete_tag(name, version))
    }

    #[pyo3(signature = (remove_tag, remove_tag_version, merge_to_tag, merge_to_version, fields=None))]
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

    pub fn get_tag_siblings(&self, name: String) -> PyResult<Vec<TagSibling>> {
        self.runtime.block_on(self.client.get_tag_siblings(name))
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
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

    #[pyo3(signature = (url=None, token=None, file_path=None, thumbnail_path=None, tags=None, safety=None, source=None,
            relations=None, notes=None, flags=None, anonymous=None, fields=None))]
    pub fn create_post(
        &self,
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
        anonymous: Option<bool>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime.block_on(self.client.create_post(
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
            anonymous,
            fields,
        ))
    }

    #[pyo3(signature = (post_id, post_version, url=None, token=None, file_path=None,
        thumbnail_path=None, tags=None, safety=None, source=None, relations=None, notes=None,
        flags=None, fields=None))]
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

    pub fn get_image_bytes<'py>(&self, post_id: u32) -> PyResult<Vec<u8>> {
        self.runtime.block_on(self.client.get_image_bytes(post_id))
    }

    pub fn download_image_to_path(&self, post_id: u32, file_path: PathBuf) -> PyResult<()> {
        self.runtime
            .block_on(self.client.download_image_to_path(post_id, file_path))
    }

    pub fn get_thumbnail_bytes<'py>(&self, post_id: u32) -> PyResult<Vec<u8>> {
        self.runtime
            .block_on(self.client.get_thumbnail_bytes(post_id))
    }

    pub fn download_thumbnail_to_path(&self, post_id: u32, file_path: PathBuf) -> PyResult<()> {
        self.runtime
            .block_on(self.client.download_thumbnail_to_path(post_id, file_path))
    }

    pub fn reverse_image_search(&self, image_path: PathBuf) -> PyResult<ImageSearchResult> {
        self.runtime
            .block_on(self.client.reverse_image_search(image_path))
    }

    pub fn post_for_image(&self, image_path: PathBuf) -> PyResult<Option<PostResource>> {
        self.runtime
            .block_on(self.client.post_for_image(image_path))
    }

    #[pyo3(signature = (post_id, fields=None))]
    pub fn get_post(&self, post_id: u32, fields: Option<Vec<String>>) -> PyResult<PostResource> {
        self.runtime.block_on(self.client.get_post(post_id, fields))
    }

    pub fn get_around_post(&self, post_id: u32) -> PyResult<AroundPostResult> {
        self.runtime.block_on(self.client.get_around_post(post_id))
    }

    pub fn delete_post(&self, post_id: u32, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_post(post_id, version))
    }

    #[pyo3(signature = (remove_post, remove_post_version, merge_to_post,
        merge_to_version, replace_post_content=false, fields=None))]
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
    pub fn favorite_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime
            .block_on(self.client.favorite_post(post_id, fields))
    }

    #[pyo3(signature = (post_id, fields=None))]
    pub fn unfavorite_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime
            .block_on(self.client.unfavorite_post(post_id, fields))
    }

    #[pyo3(signature = (fields=None))]
    pub fn get_featured_post(&self, fields: Option<Vec<String>>) -> PyResult<Option<PostResource>> {
        self.runtime.block_on(self.client.get_featured_post(fields))
    }

    #[pyo3(signature = (post_id, fields=None))]
    pub fn set_featured_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.runtime
            .block_on(self.client.set_featured_post(post_id, fields))
    }

    #[pyo3(signature = (fields=None))]
    pub fn list_pool_categories(
        &self,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<PoolCategoryResource>> {
        self.runtime
            .block_on(self.client.list_pool_categories(fields))
    }

    #[pyo3(signature = (name, color=None, fields=None))]
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
    pub fn get_pool_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        self.runtime
            .block_on(self.client.get_pool_category(name, fields))
    }

    pub fn delete_pool_category(&self, name: String, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_pool_category(name, version))
    }

    #[pyo3(signature = (name, fields=None))]
    pub fn set_default_pool_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        self.runtime
            .block_on(self.client.set_default_pool_category(name, fields))
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
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

    #[pyo3(signature = (pool_id, version, names=None, category=None, description=None,
        posts=None, fields=None))]
    pub fn update_pool(
        &self,
        pool_id: u32,
        version: u32,
        names: Option<Vec<String>>,
        category: Option<String>,
        description: Option<String>,
        posts: Option<Vec<u32>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        self.runtime.block_on(self.client.update_pool(
            pool_id,
            version,
            names,
            category,
            description,
            posts,
            fields,
        ))
    }

    #[pyo3(signature = (pool_id, fields=None))]
    pub fn get_pool(&self, pool_id: u32, fields: Option<Vec<String>>) -> PyResult<PoolResource> {
        self.runtime.block_on(self.client.get_pool(pool_id, fields))
    }

    pub fn delete_pool(&self, pool_id: u32, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_pool(pool_id, version))
    }

    #[pyo3(signature = (remove_pool, remove_pool_version, merge_to_pool, merge_to_version, fields=None))]
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
    pub fn get_comment(
        &self,
        comment_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        self.runtime
            .block_on(self.client.get_comment(comment_id, fields))
    }

    pub fn delete_comment(&self, comment_id: u32, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_comment(comment_id, version))
    }

    #[pyo3(signature = (comment_id, rating, fields=None))]
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
    pub fn get_user(
        &self,
        user_name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserResource> {
        self.runtime
            .block_on(self.client.get_user(user_name, fields))
    }

    pub fn delete_user(&self, user_name: String, version: u32) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_user(user_name, version))
    }

    #[pyo3(signature = (user_name, fields=None))]
    pub fn list_user_tokens(
        &self,
        user_name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<UserAuthTokenResource>> {
        self.runtime
            .block_on(self.client.list_user_tokens(user_name, fields))
    }

    #[pyo3(signature = (user_name, note=None, enabled=None, expiration_time=None, fields=None))]
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

    pub fn delete_user_token(
        &self,
        user_name: String,
        token: String,
        version: u32,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.client.delete_user_token(user_name, token, version))
    }

    pub fn password_reset_request(&self, email_or_name: String) -> PyResult<()> {
        self.runtime
            .block_on(self.client.password_reset_request(email_or_name))
    }

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

    pub fn global_info(&self) -> PyResult<GlobalInfo> {
        self.runtime.block_on(self.client.global_info())
    }

    pub fn upload_temporary_file(&self, file_path: PathBuf) -> PyResult<String> {
        self.runtime
            .block_on(self.client.upload_temporary_file(file_path))
    }
}
