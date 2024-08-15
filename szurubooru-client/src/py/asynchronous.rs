use crate::models::*;
use crate::py::PyPagedSearchResult;
use crate::tokens::QueryToken;
use crate::SzurubooruClient;
use chrono::{DateTime, Utc};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(name = "SzurubooruAsyncClient")]
pub struct PythonAsyncClient {
    client: SzurubooruClient,
}

#[pymethods]
impl PythonAsyncClient {
    #[new]
    #[pyo3(signature = (host, username=None, token=None, password=None, allow_insecure=None))]
    pub fn new(
        host: String,
        username: Option<String>,
        token: Option<String>,
        password: Option<String>,
        allow_insecure: Option<bool>,
    ) -> PyResult<Self> {
        let allow_insecure = allow_insecure.unwrap_or(false);

        match (username, token, password) {
            (Some(u), Some(t), None) => {
                let client = SzurubooruClient::new_with_token(&host, &u, &t, allow_insecure)?;
                Ok(PythonAsyncClient { client })
            }
            (Some(u), None, Some(p)) => {
                let client = SzurubooruClient::new_with_basic_auth(&host, &u, &p, allow_insecure)?;
                Ok(PythonAsyncClient { client })
            }
            (None, None, None) => {
                let client = SzurubooruClient::new_anonymous(&host, allow_insecure)?;
                Ok(PythonAsyncClient { client })
            }
            _ => Err(PyRuntimeError::new_err(
                "(Username and Token) or (Username and Password) must be provided",
            )),
        }
    }

    #[pyo3(signature = (fields=None))]
    pub async fn list_tag_categories(
        &self,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<TagCategoryResource>> {
        let request = self.client.with_optional_fields(fields);
        request
            .list_tag_categories()
            .await
            .map(|ltc| ltc.results)
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, version, color=None, order=None, fields=None))]
    pub async fn update_tag_category(
        &self,
        name: String,
        version: u32,
        color: Option<String>,
        order: Option<u32>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        let mut cutag = CreateUpdateTagCategoryBuilder::default();
        let mut cutag = cutag.version(version);

        if let Some(color) = color {
            cutag = cutag.color(color);
        }
        if let Some(order) = order {
            cutag = cutag.order(order);
        }

        let cutag = cutag.build()?;
        let request = self.client.with_optional_fields(fields);
        request
            .update_tag_category(name, &cutag)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, fields=None))]
    pub async fn get_tag_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        self.client
            .with_optional_fields(fields)
            .get_tag_category(name)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_tag_category(&self, name: String, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_tag_category(name, version)
            .await
            .map_err(Into::into)
    }

    pub async fn set_default_tag_category(&self, name: String) -> PyResult<()> {
        self.client
            .request()
            .set_default_tag_category(name)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    pub async fn list_tags(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.client
            .with_optional_fields(fields)
            .with_optional_limit(limit)
            .with_optional_offset(offset)
            .list_tags(query.as_ref())
            .await
            .map_err(Into::into)
            .map(Into::into)
    }

    #[pyo3(signature = (names, category=None, description=None, implications=None, suggestions=None, fields=None))]
    pub async fn create_tag(
        &self,
        names: Vec<String>,
        category: Option<String>,
        description: Option<String>,
        implications: Option<Vec<String>>,
        suggestions: Option<Vec<String>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagResource> {
        let mut cubuild = CreateUpdateTagBuilder::default();
        cubuild.names(names);
        if let Some(cat) = category {
            cubuild.category(cat);
        }
        if let Some(desc) = description {
            cubuild.description(desc);
        }
        if let Some(imps) = implications {
            cubuild.implications(imps);
        }
        if let Some(s) = suggestions {
            cubuild.suggestions(s);
        }
        let tag_build = cubuild.build()?;
        self.client
            .with_optional_fields(fields)
            .create_tag(&tag_build)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, names, category=None, description=None, implications=None, suggestions=None, fields=None))]
    pub async fn update_tag(
        &self,
        name: String,
        names: Option<Vec<String>>,
        category: Option<String>,
        description: Option<String>,
        implications: Option<Vec<String>>,
        suggestions: Option<Vec<String>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagResource> {
        let mut cubuild = CreateUpdateTagBuilder::default();
        if let Some(names) = names {
            cubuild.names(names);
        }
        if let Some(cat) = category {
            cubuild.category(cat);
        }
        if let Some(desc) = description {
            cubuild.description(desc);
        }
        if let Some(imps) = implications {
            cubuild.implications(imps);
        }
        if let Some(s) = suggestions {
            cubuild.suggestions(s);
        }
        let tag_build = cubuild.build()?;
        self.client
            .with_optional_fields(fields)
            .update_tag(name, &tag_build)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, fields=None))]
    pub async fn get_tag(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagResource> {
        self.client
            .with_optional_fields(fields)
            .get_tag(name)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_tag(&self, name: String, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_tag(name, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (remove_tag, remove_tag_version, merge_to_tag, merge_to_version, fields=None))]
    pub async fn merge_tag(
        &self,
        remove_tag: String,
        remove_tag_version: u32,
        merge_to_tag: String,
        merge_to_version: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagResource> {
        let mtags = MergeTagsBuilder::default()
            .remove_tag_version(remove_tag_version)
            .remove_tag(remove_tag)
            .merge_to_version(merge_to_version)
            .merge_to_tag(merge_to_tag)
            .build()?;
        self.client
            .with_optional_fields(fields)
            .merge_tag(&mtags)
            .await
            .map_err(Into::into)
    }

    pub async fn get_tag_siblings(&self, name: String) -> PyResult<Vec<TagSibling>> {
        self.client
            .request()
            .get_tag_siblings(name)
            .await
            .map(|ts| ts.results)
            .map_err(Into::into)
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    pub async fn list_posts(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.client
            .with_optional_fields(fields)
            .with_optional_limit(limit)
            .with_optional_offset(offset)
            .list_posts(query.as_ref())
            .await
            .map_err(Into::into)
            .map(Into::into)
    }

    #[pyo3(signature = (url=None, token=None, file_path=None, thumbnail_path=None, tags=None, safety=None, source=None,
            relations=None, notes=None, flags=None, fields=None))]
    pub async fn create_post(
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
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        let mut cupost = CreateUpdatePostBuilder::default();
        if let Some(source) = source {
            cupost.source(source);
        }
        if let Some(tags) = tags {
            cupost.tags(tags);
        }
        if let Some(safety) = safety {
            cupost.safety(safety);
        }
        if let Some(relations) = relations {
            cupost.relations(relations);
        }
        if let Some(notes) = notes {
            cupost.notes(notes);
        }
        if let Some(flags) = flags {
            cupost.flags(flags);
        }

        if let Some(token) = token {
            cupost.content_token(token);
            let cupost = cupost.build()?;
            self.client
                .with_optional_fields(fields)
                .create_post_from_token(&cupost)
                .await
                .map_err(Into::into)
        } else if let Some(url) = url {
            cupost.content_url(url);
            let cupost = cupost.build()?;
            self.client
                .with_optional_fields(fields)
                .create_post_from_url(&cupost)
                .await
                .map_err(Into::into)
        } else if let Some(file) = file_path {
            let cupost = cupost.build()?;
            self.client
                .with_optional_fields(fields)
                .create_post_from_file_path(file, thumbnail_path, &cupost)
                .await
                .map_err(Into::into)
        } else {
            Err(PyRuntimeError::new_err(
                "One of url, token or file must be specified",
            ))
        }
    }

    #[pyo3(signature = (post_id, post_version, url=None, token=None, file_path=None,
        thumbnail_path=None, tags=None, safety=None, source=None, relations=None, notes=None,
        flags=None, fields=None))]
    pub async fn update_post(
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
        let mut cupost = CreateUpdatePostBuilder::default();
        cupost.version(post_version);
        if let Some(source) = source {
            cupost.source(source);
        }
        if let Some(tags) = tags {
            cupost.tags(tags);
        }
        if let Some(safety) = safety {
            cupost.safety(safety);
        }
        if let Some(relations) = relations {
            cupost.relations(relations);
        }
        if let Some(notes) = notes {
            cupost.notes(notes);
        }
        if let Some(flags) = flags {
            cupost.flags(flags);
        }

        if let Some(token) = token {
            cupost.content_token(token);
            let cupost = cupost.build()?;
            self.client
                .with_optional_fields(fields)
                .update_post_from_token(post_id, &cupost)
                .await
                .map_err(Into::into)
        } else if let Some(url) = url {
            cupost.content_url(url);
            let cupost = cupost.build()?;
            self.client
                .with_optional_fields(fields)
                .update_post_from_url(post_id, &cupost)
                .await
                .map_err(Into::into)
        } else if file_path.is_some() || thumbnail_path.is_some() {
            let cupost = cupost.build()?;
            self.client
                .with_optional_fields(fields)
                .update_post_from_file_path(post_id, file_path, thumbnail_path, &cupost)
                .await
                .map_err(Into::into)
        } else {
            let cupost = cupost.build()?;
            self.client
                .with_optional_fields(fields)
                .update_post(post_id, &cupost)
                .await
                .map_err(Into::into)
        }
    }

    pub async fn get_image_bytes(&self, post_id: u32) -> PyResult<Vec<u8>> {
        let bytes = self
            .client
            .request()
            .get_image_bytes(post_id)
            .await?
            .to_vec();
        Ok(bytes)
    }

    pub async fn download_image_to_path(&self, post_id: u32, file_path: PathBuf) -> PyResult<()> {
        self.client
            .request()
            .download_image_to_path(post_id, file_path)
            .await
            .map_err(Into::into)
    }

    pub async fn get_thumbnail_bytes<'py>(&self, post_id: u32) -> PyResult<Vec<u8>> {
        let bytes = self
            .client
            .request()
            .get_thumbnail_bytes(post_id)
            .await?
            .to_vec();
        Ok(bytes)
    }

    pub async fn download_thumbnail_to_path(
        &self,
        post_id: u32,
        file_path: PathBuf,
    ) -> PyResult<()> {
        self.client
            .request()
            .download_thumbnail_to_path(post_id, file_path)
            .await
            .map_err(Into::into)
    }

    pub async fn reverse_search_image(&self, image_path: PathBuf) -> PyResult<ImageSearchResult> {
        self.client
            .request()
            .reverse_search_file_path(image_path)
            .await
            .map_err(Into::into)
    }

    pub async fn post_for_image(&self, image_path: PathBuf) -> PyResult<Option<PostResource>> {
        self.client
            .request()
            .posts_for_file_path(image_path)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (post_id, fields=None))]
    pub async fn get_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.client
            .with_optional_fields(fields)
            .get_post(post_id)
            .await
            .map_err(Into::into)
    }

    pub async fn get_around_post(&self, post_id: u32) -> PyResult<AroundPostResult> {
        self.client
            .request()
            .get_around_post(post_id)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_post(&self, post_id: u32, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_post(post_id, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (remove_post, remove_post_version, merge_to_post,
        merge_to_version, fields=None))]
    pub async fn merge_post(
        &self,
        remove_post: u32,
        remove_post_version: u32,
        merge_to_post: u32,
        merge_to_version: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        let mpost = MergePostBuilder::default()
            .remove_post_version(remove_post_version)
            .remove_post(remove_post)
            .merge_to_version(merge_to_version)
            .merge_to_post(merge_to_post)
            .build()?;
        self.client
            .with_optional_fields(fields)
            .merge_post(&mpost)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (post_id, rating, fields=None))]
    pub async fn rate_post(
        &self,
        post_id: u32,
        rating: i8,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        if rating < -1 || rating > 1 {
            Err(PyValueError::new_err("Rating must be -1, 0, or 1"))
        } else {
            self.client
                .with_optional_fields(fields)
                .rate_post(post_id, rating)
                .await
                .map_err(Into::into)
        }
    }

    #[pyo3(signature = (post_id, fields=None))]
    pub async fn favorite_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.client
            .with_optional_fields(fields)
            .favorite_post(post_id)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (post_id, fields=None))]
    pub async fn unfavorite_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.client
            .with_optional_fields(fields)
            .unfavorite_post(post_id)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (fields=None))]
    pub async fn get_featured_post(
        &self,
        fields: Option<Vec<String>>,
    ) -> PyResult<Option<PostResource>> {
        self.client
            .with_optional_fields(fields)
            .get_featured_post()
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (post_id, fields=None))]
    pub async fn set_featured_post(
        &self,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        self.client
            .with_optional_fields(fields)
            .set_featured_post(post_id)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (fields=None))]
    pub async fn list_pool_categories(
        &self,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<PoolCategoryResource>> {
        self.client
            .with_optional_fields(fields)
            .list_pool_categories()
            .await
            .map_err(Into::into)
            .map(|pc| pc.results)
    }

    #[pyo3(signature = (name, color=None, fields=None))]
    pub async fn create_pool_category(
        &self,
        name: String,
        color: Option<String>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        let mut pc = CreateUpdatePoolCategoryBuilder::default();
        pc.name(name);
        if let Some(color) = color {
            pc.color(color);
        }
        let pc = pc.build()?;
        self.client
            .with_optional_fields(fields)
            .create_pool_category(&pc)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, version, new_name=None, color=None, fields=None))]
    pub async fn update_pool_category(
        &self,
        name: String,
        version: u32,
        new_name: Option<String>,
        color: Option<String>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        let mut pc = CreateUpdatePoolCategoryBuilder::default();
        pc.version(version);
        if let Some(name) = new_name {
            pc.name(name);
        }
        if let Some(color) = color {
            pc.color(color);
        }
        let pc = pc.build()?;
        self.client
            .with_optional_fields(fields)
            .update_pool_category(name, &pc)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, fields=None))]
    pub async fn get_pool_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        self.client
            .with_optional_fields(fields)
            .get_pool_category(name)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_pool_category(&self, name: String, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_pool_category(name, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, fields=None))]
    pub async fn set_default_pool_category(
        &self,
        name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolCategoryResource> {
        self.client
            .with_optional_fields(fields)
            .set_default_pool_category(name)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    pub async fn list_pools(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.client
            .with_optional_fields(fields)
            .with_optional_limit(limit)
            .with_optional_offset(offset)
            .list_pools(query.as_ref())
            .await
            .map_err(Into::into)
            .map(Into::into)
    }

    #[pyo3(signature = (names, category=None, description=None, posts=None, fields=None))]
    pub async fn create_pool<'py>(
        &self,
        names: Vec<String>,
        category: Option<String>,
        description: Option<String>,
        posts: Option<Vec<u32>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        let mut cupool = CreateUpdatePoolBuilder::default();
        cupool.names(names);
        if let Some(cat) = category {
            cupool.category(cat);
        }
        if let Some(desc) = description {
            cupool.description(desc);
        }
        if let Some(posts) = posts {
            cupool.posts(posts);
        }
        let cupool = cupool.build()?;
        self.client
            .with_optional_fields(fields)
            .create_pool(&cupool)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (pool_id, version, names=None, category=None, description=None,
        posts=None, fields=None))]
    pub async fn update_pool(
        &self,
        pool_id: u32,
        version: u32,
        names: Option<Vec<String>>,
        category: Option<String>,
        description: Option<String>,
        posts: Option<Vec<u32>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        let mut cupool = CreateUpdatePoolBuilder::default();
        cupool.version(version);
        if let Some(names) = names {
            cupool.names(names);
        }

        if let Some(cat) = category {
            cupool.category(cat);
        }
        if let Some(desc) = description {
            cupool.description(desc);
        }
        if let Some(posts) = posts {
            cupool.posts(posts);
        }
        let cupool = cupool.build()?;
        self.client
            .with_optional_fields(fields)
            .update_pool(pool_id, &cupool)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (pool_id, fields=None))]
    pub async fn get_pool(
        &self,
        pool_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        self.client
            .with_optional_fields(fields)
            .get_pool(pool_id)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_pool(&self, pool_id: u32, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_pool(pool_id, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (remove_pool, remove_pool_version, merge_to_pool, merge_to_version, fields=None))]
    pub async fn merge_pools(
        &self,
        remove_pool: u32,
        remove_pool_version: u32,
        merge_to_pool: u32,
        merge_to_version: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        let mpool = MergePoolBuilder::default()
            .remove_pool_version(remove_pool_version)
            .remove_pool(remove_pool)
            .merge_to_version(merge_to_version)
            .merge_to_pool(merge_to_pool)
            .build()?;
        self.client
            .with_optional_fields(fields)
            .merge_pools(&mpool)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    pub async fn list_comments(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.client
            .with_optional_fields(fields)
            .with_optional_limit(limit)
            .with_optional_offset(offset)
            .list_comments(query.as_ref())
            .await
            .map_err(Into::into)
            .map(Into::into)
    }

    #[pyo3(signature = (text, post_id, fields=None))]
    pub async fn create_comment(
        &self,
        text: String,
        post_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        let mut cucomment = CreateUpdateCommentBuilder::default();
        cucomment.post_id(post_id);
        cucomment.text(text);

        let cucomment = cucomment.build()?;
        self.client
            .with_optional_fields(fields)
            .create_comment(&cucomment)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (comment_id, version, text, fields=None))]
    pub async fn update_comment(
        &self,
        comment_id: u32,
        version: u32,
        text: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        let mut cucomment = CreateUpdateCommentBuilder::default();
        cucomment.version(version);
        cucomment.text(text);

        let cucomment = cucomment.build()?;
        self.client
            .with_optional_fields(fields)
            .update_comment(comment_id, &cucomment)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (comment_id, fields=None))]
    pub async fn get_comment(
        &self,
        comment_id: u32,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        self.client
            .with_optional_fields(fields)
            .get_comment(comment_id)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_comment(&self, comment_id: u32, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_comment(comment_id, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (comment_id, rating, fields=None))]
    pub async fn rate_comment(
        &self,
        comment_id: u32,
        rating: i8,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        if rating < -1 || rating > 1 {
            Err(PyValueError::new_err("Rating must be -1, 0, or 1"))
        } else {
            self.client
                .with_optional_fields(fields)
                .rate_comment(comment_id, rating)
                .await
                .map_err(Into::into)
        }
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    pub async fn list_users(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.client
            .with_optional_fields(fields)
            .with_optional_limit(limit)
            .with_optional_offset(offset)
            .list_users(query.as_ref())
            .await
            .map_err(Into::into)
            .map(Into::into)
    }

    #[pyo3(signature = (name, password, rank=None, avatar_path=None, fields=None))]
    pub async fn create_user(
        &self,
        name: String,
        password: String,
        rank: Option<UserRank>,
        avatar_path: Option<PathBuf>,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserResource> {
        let mut cuser = CreateUpdateUserBuilder::default();
        cuser.name(name);
        cuser.password(password);
        if let Some(rank) = rank {
            cuser.rank(rank);
        }
        if let Some(avatar_path) = avatar_path {
            cuser.avatar_style(UserAvatarStyle::Manual);
            let cuser = cuser.build()?;
            self.client
                .with_optional_fields(fields)
                .create_user_with_avatar_path(avatar_path, &cuser)
                .await
                .map_err(Into::into)
        } else {
            cuser.avatar_style(UserAvatarStyle::Gravatar);
            let cuser = cuser.build()?;
            self.client
                .with_optional_fields(fields)
                .create_user(&cuser)
                .await
                .map_err(Into::into)
        }
    }

    #[pyo3(signature = (name, version, new_name=None, password=None, rank=None, avatar_path=None, fields=None))]
    pub async fn update_user(
        &self,
        name: String,
        version: u32,
        new_name: Option<String>,
        password: Option<String>,
        rank: Option<UserRank>,
        avatar_path: Option<PathBuf>,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserResource> {
        let mut cuser = CreateUpdateUserBuilder::default();
        cuser.version(version);
        if let Some(new_name) = new_name {
            cuser.name(new_name);
        }
        if let Some(password) = password {
            cuser.password(password);
        }
        if let Some(rank) = rank {
            cuser.rank(rank);
        }
        if let Some(avatar_path) = avatar_path {
            cuser.avatar_style(UserAvatarStyle::Manual);
            let cuser = cuser.build()?;
            self.client
                .with_optional_fields(fields)
                .update_user_with_avatar_path(name, avatar_path, &cuser)
                .await
                .map_err(Into::into)
        } else {
            cuser.avatar_style(UserAvatarStyle::Gravatar);
            let cuser = cuser.build()?;
            self.client
                .with_optional_fields(fields)
                .update_user(name, &cuser)
                .await
                .map_err(Into::into)
        }
    }

    #[pyo3(signature = (user_name, fields=None))]
    pub async fn get_user(
        &self,
        user_name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserResource> {
        self.client
            .with_optional_fields(fields)
            .get_user(user_name)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_user(&self, user_name: String, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_user(user_name, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (user_name, fields=None))]
    pub async fn list_user_tokens(
        &self,
        user_name: String,
        fields: Option<Vec<String>>,
    ) -> PyResult<Vec<UserAuthTokenResource>> {
        self.client
            .with_optional_fields(fields)
            .list_user_tokens(user_name)
            .await
            .map_err(Into::into)
            .map(|ur| ur.results)
    }

    #[pyo3(signature = (user_name, note=None, expiration_time=None, fields=None))]
    pub async fn create_user_token(
        &self,
        user_name: String,
        note: Option<String>,
        expiration_time: Option<DateTime<Utc>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserAuthTokenResource> {
        let mut cutoken = CreateUpdateUserAuthTokenBuilder::default();
        if let Some(note) = note {
            cutoken.note(note);
        }
        if let Some(etime) = expiration_time {
            cutoken.expiration_time(etime);
        }
        let cutoken = cutoken.build()?;
        self.client
            .with_optional_fields(fields)
            .create_user_token(user_name, &cutoken)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (user_name, token, version, enabled=None, note=None, expiration_time=None, fields=None))]
    pub async fn update_user_token(
        &self,
        user_name: String,
        token: String,
        version: u32,
        enabled: Option<bool>,
        note: Option<String>,
        expiration_time: Option<DateTime<Utc>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<UserAuthTokenResource> {
        let mut cutoken = CreateUpdateUserAuthTokenBuilder::default();
        cutoken.version(version);
        if let Some(enabled) = enabled {
            cutoken.enabled(enabled);
        }
        if let Some(note) = note {
            cutoken.note(note);
        }
        if let Some(etime) = expiration_time {
            cutoken.expiration_time(etime);
        }
        let cutoken = cutoken.build()?;
        self.client
            .with_optional_fields(fields)
            .update_user_token(user_name, token, &cutoken)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_user_token(
        &self,
        user_name: String,
        token: String,
        version: u32,
    ) -> PyResult<()> {
        self.client
            .request()
            .delete_user_token(user_name, token, version)
            .await
            .map_err(Into::into)
    }

    pub async fn password_reset_request(&self, email_or_name: String) -> PyResult<()> {
        self.client
            .request()
            .password_reset_request(email_or_name)
            .await
            .map_err(Into::into)
    }

    pub async fn password_reset_confirm(
        &self,
        email_or_name: String,
        reset_token: String,
    ) -> PyResult<String> {
        self.client
            .request()
            .password_reset_confirm(email_or_name, reset_token)
            .await
            .map_err(Into::into)
            .map(|tp| tp.password)
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    pub async fn list_snapshots(
        &self,
        query: Option<Vec<QueryToken>>,
        fields: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> PyResult<PyPagedSearchResult> {
        self.client
            .with_optional_fields(fields)
            .with_optional_limit(limit)
            .with_optional_offset(offset)
            .list_snapshots(query.as_ref())
            .await
            .map_err(Into::into)
            .map(Into::into)
    }

    pub async fn global_info(&self) -> PyResult<GlobalInfo> {
        self.client
            .request()
            .get_global_info()
            .await
            .map_err(Into::into)
    }

    pub async fn upload_temporary_file(&self, file_path: PathBuf) -> PyResult<TemporaryFileUpload> {
        self.client
            .request()
            .upload_temporary_file_from_path(file_path)
            .await
            .map_err(Into::into)
    }
}
