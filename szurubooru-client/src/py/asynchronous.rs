use crate::models::*;
use crate::py::PyPagedSearchResult;
use crate::tokens::QueryToken;
use crate::SzurubooruClient;
use chrono::{DateTime, Utc};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(name = "SzurubooruAsyncClient", module = "szurubooru_client")]
/// An asynchronous client for Szurubooru
///
/// :see: :class:`~szurubooru_client.SzurubooruSyncClient` for supported parameters
pub struct PythonAsyncClient {
    client: SzurubooruClient,
}

#[pymethods]
impl PythonAsyncClient {
    #[new]
    #[pyo3(signature = (host, username=None, token=None, password=None, allow_insecure=None))]
    /// Creates a new instance of the Asynchornous client
    ///
    /// :see: :class:`~szurubooru_client.SzurubooruSyncClient` for supported parameters
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
    /// List the available tag categories (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_tag_categories` for parameters and return type
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

    #[pyo3(signature = (name, color=None, order=None, fields=None))]
    /// Creates a new tag category using the specified parameters (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.create_tag_category` for parameters and return type
    pub async fn create_tag_category(
        &self,
        name: String,
        color: Option<String>,
        order: Option<u32>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        let mut cutagcat = CreateUpdateTagCategoryBuilder::default();
        cutagcat.name(name);
        if let Some(color) = color {
            cutagcat.color(color);
        }
        if let Some(order) = order {
            cutagcat.order(order);
        }
        let cutagcat = cutagcat.build()?;
        self.client
            .with_optional_fields(fields)
            .create_tag_category(&cutagcat)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, version, new_name=None, color=None, order=None, fields=None))]
    /// Updates an existing tag category (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.update_tag_category` for parameters and return type
    pub async fn update_tag_category(
        &self,
        name: String,
        version: u32,
        new_name: Option<String>,
        color: Option<String>,
        order: Option<u32>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagCategoryResource> {
        let mut cutag = CreateUpdateTagCategoryBuilder::default();
        let mut cutag = cutag.version(version);

        if let Some(name) = new_name {
            cutag = cutag.name(name);
        }
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
    /// Fetches a tag category by name (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_tag_category` for parameters and return type
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

    #[pyo3(signature = (name, version))]
    /// Deletes a tag category (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.delete_tag_category` for parameters and return type
    pub async fn delete_tag_category(&self, name: String, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_tag_category(name, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name))]
    /// Sets the default tag category for the site (async version)
    pub async fn set_default_tag_category(&self, name: String) -> PyResult<()> {
        self.client
            .request()
            .set_default_tag_category(name)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// List the tags currently available on the site (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_tags` for parameters and return type
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
    /// Creating a new tag (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.create_tag` for parameters and return type
    pub async fn create_tag(
        &self,
        names: Py<PyAny>,
        category: Option<String>,
        description: Option<String>,
        implications: Option<Vec<String>>,
        suggestions: Option<Vec<String>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<TagResource> {
        let mut cubuild = CreateUpdateTagBuilder::default();
        Python::with_gil(|py| {
            if let Ok(name) = names.extract::<String>(py) {
                Ok(cubuild.names(vec![name]))
            } else {
                let list_res = names.extract::<Vec<String>>(py);
                if let Ok(names) = list_res {
                    Ok(cubuild.names(names))
                } else {
                    Err(list_res.err().unwrap())
                }
            }
        })?;
        //cubuild.names(names);
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

    #[pyo3(signature = (name, version, names=None, category=None, description=None, implications=None, suggestions=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Updates an existing tag (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.update_tag` for parameters and return type
    pub async fn update_tag(
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
        let mut cubuild = CreateUpdateTagBuilder::default();
        cubuild.version(version);
        if let Some(names) = names {
            Python::with_gil(|py| {
                if let Ok(name) = names.extract::<String>(py) {
                    Ok(cubuild.names(vec![name]))
                } else {
                    let list_res = names.extract::<Vec<String>>(py);
                    if let Ok(names) = list_res {
                        Ok(cubuild.names(names))
                    } else {
                        Err(list_res.err().unwrap())
                    }
                }
            })?;
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
    /// Fetches an existing tag (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_tag` for parameters and return type
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

    #[pyo3(signature = (name, version))]
    /// Deletes an existing tag (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.delete_tag` for parameters and return type
    pub async fn delete_tag(&self, name: String, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_tag(name, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (remove_tag, remove_tag_version, merge_to_tag, merge_to_version, fields=None))]
    /// Removes source tag and merges all of its usages, suggestions and implications to the
    /// target tag. (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.merge_tags` for parameters and return type
    pub async fn merge_tags(
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
            .merge_tags(&mtags)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name))]
    /// Lists siblings of given tag, e.g. tags that were used in the same posts as the given tag.
    /// (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_tag_siblings` for parameters and return type
    pub async fn get_tag_siblings(&self, name: String) -> PyResult<Vec<TagSibling>> {
        self.client
            .request()
            .get_tag_siblings(name)
            .await
            .map(|ts| ts.results)
            .map_err(Into::into)
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// Lists the posts currently available on the site (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_posts` for parameters and return type
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

    #[pyo3(signature = (url=None, upload_token=None, file_path=None, thumbnail_path=None, tags=None, safety=None, source=None,
            relations=None, notes=None, flags=None, anonymous=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Create a new post using one of three image sources (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.create_post` for parameters and return type
    pub async fn create_post(
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
        if let Some(anonymous) = anonymous {
            cupost.anonymous(anonymous);
        }

        if let Some(token) = upload_token {
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
    #[allow(clippy::too_many_arguments)]
    /// Updates an existing post (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.update_post` for parameters and return type
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

    #[pyo3(signature = (post_id))]
    /// Downloads the given post's image as a byte array (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_image_bytes` for parameters and return type
    pub async fn get_image_bytes(&self, post_id: u32) -> PyResult<Vec<u8>> {
        let bytes = self
            .client
            .request()
            .get_image_bytes(post_id)
            .await?
            .to_vec();
        Ok(bytes)
    }

    #[pyo3(signature = (post_id, file_path))]
    /// Downloads the given post's image to a path on the filesystem
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.download_image_to_path` for parameters and return type
    pub async fn download_image_to_path(&self, post_id: u32, file_path: PathBuf) -> PyResult<()> {
        self.client
            .request()
            .download_image_to_path(post_id, file_path)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (post_id))]
    /// Downloads the given post's thumbnail as a byte array
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_thumbnail_bytes` for parameters and return type
    pub async fn get_thumbnail_bytes<'py>(&self, post_id: u32) -> PyResult<Vec<u8>> {
        let bytes = self
            .client
            .request()
            .get_thumbnail_bytes(post_id)
            .await?
            .to_vec();
        Ok(bytes)
    }

    #[pyo3(signature = (post_id, file_path))]
    /// Downloads the given post's thumbnail to a path on the filesystem
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.` for parameters and return type
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

    #[pyo3(signature = (image_path))]
    /// Reverse image searches for an image from the filesystem (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.reverse_image_search` for parameters and return type
    pub async fn reverse_image_search(&self, image_path: PathBuf) -> PyResult<ImageSearchResult> {
        self.client
            .request()
            .reverse_search_file_path(image_path)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (image_path))]
    /// Searches for an *exact* image match of an image from the filesystem (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.post_for_image` for parameters and return type
    pub async fn post_for_image(&self, image_path: PathBuf) -> PyResult<Option<PostResource>> {
        self.client
            .request()
            .post_for_file_path(image_path)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (post_id, fields=None))]
    /// Fetches an individual post by its post ID (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_post` for parameters and return type
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

    /// Fetches posts from *around* the given post ID. That means the post before and after,
    //  if they exist. (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_around_post` for parameters and return type
    pub async fn get_around_post(&self, post_id: u32) -> PyResult<AroundPostResult> {
        self.client
            .request()
            .get_around_post(post_id)
            .await
            .map_err(Into::into)
    }

    /// Deletes a post by its ID (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.delete_post` for parameters and return type
    pub async fn delete_post(&self, post_id: u32, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_post(post_id, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (remove_post, remove_post_version, merge_to_post,
        merge_to_version, replace_post_content=false, fields=None))]
    /// Removes source post and merges all of its tags, relations, scores, favorites and comments to
    /// the target post (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.merge_post` for parameters and return type
    pub async fn merge_post(
        &self,
        remove_post: u32,
        remove_post_version: u32,
        merge_to_post: u32,
        merge_to_version: u32,
        replace_post_content: bool,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        let mpost = MergePostBuilder::default()
            .remove_post_version(remove_post_version)
            .remove_post(remove_post)
            .merge_to_version(merge_to_version)
            .merge_to_post(merge_to_post)
            .replace_post_content(replace_post_content)
            .build()?;
        self.client
            .with_optional_fields(fields)
            .merge_post(&mpost)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (post_id, rating, fields=None))]
    /// Updates score of authenticated user for given post. Valid scores are -1, 0 and 1.
    /// (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.rate_post` for parameters and return type
    pub async fn rate_post(
        &self,
        post_id: u32,
        rating: i8,
        fields: Option<Vec<String>>,
    ) -> PyResult<PostResource> {
        if !(-1..=1).contains(&rating) {
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
    /// Marks the post as favorite for the current user (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.favorite_post` for parameters and return type
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
    /// Unmarks the post as favorite for the current user (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.unfavorite_post` for parameters and return type
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
    /// Retrieves the post that is currently featured on the main page (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_featured_post` for parameters and return type
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
    /// Features a post on the main page (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.set_featured_post` for parameters and return type
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
    /// Lists all pool categories (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_pool_categories` for parameters and return type
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
    /// Creates a new pool category using specified parameters (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.create_pool_category` for parameters and return type
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
    /// Updates an existing tag category using specified parameters (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.update_pool_category` for parameters and return type
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
    /// Fetches an existing pool category (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_pool_category` for parameters and return type
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

    /// Deletes existing pool category (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.delete_pool_category` for parameters and return type
    pub async fn delete_pool_category(&self, name: String, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_pool_category(name, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (name, fields=None))]
    /// Sets given pool category as default (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.set_default_pool_category` for parameters and return type
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
    /// List the post pools currently available on the site (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_pools` for parameters and return type
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
    /// Creates a new pool using specified parameters (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.create_pool` for parameters and return type
    pub async fn create_pool<'py>(
        &self,
        names: Py<PyAny>,
        category: Option<String>,
        description: Option<String>,
        posts: Option<Vec<u32>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        let mut cupool = CreateUpdatePoolBuilder::default();
        Python::with_gil(|py| {
            if let Ok(name) = names.extract::<String>(py) {
                Ok(cupool.names(vec![name]))
            } else {
                let list_res = names.extract::<Vec<String>>(py);
                if let Ok(names) = list_res {
                    Ok(cupool.names(names))
                } else {
                    Err(list_res.err().unwrap())
                }
            }
        })?;
        //cupool.names(names);
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

    #[pyo3(signature = (pool_id, version, new_names=None, category=None, description=None,
        posts=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Updates an existing pool using specified parameters (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.update_pool` for parameters and return type
    pub async fn update_pool(
        &self,
        pool_id: u32,
        version: u32,
        new_names: Option<Vec<String>>,
        category: Option<String>,
        description: Option<String>,
        posts: Option<Vec<u32>>,
        fields: Option<Vec<String>>,
    ) -> PyResult<PoolResource> {
        let mut cupool = CreateUpdatePoolBuilder::default();
        cupool.version(version);
        if let Some(names) = new_names {
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
    /// Retrieves information about an existing pool (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_pool` for parameters and return type
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

    /// Deletes existing pool (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.delete_pool` for parameters and return type
    pub async fn delete_pool(&self, pool_id: u32, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_pool(pool_id, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (remove_pool, remove_pool_version, merge_to_pool, merge_to_version, fields=None))]
    /// Removes source pool and merges all of its posts with the target pool. (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.merge_pools` for parameters and return type
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
    /// List the comments currently available on the site (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_comments` for parameters and return type
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
    /// Creates a new comment under a given post (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.create_comment` for parameters and return type
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
    /// Updates an existing comment with new text (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.update_comment` for parameters and return type
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
    /// Fetches an existing comment (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_comment` for parameters and return type
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

    /// Deletes an existing comment (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.delete_comment` for parameters and return type
    pub async fn delete_comment(&self, comment_id: u32, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_comment(comment_id, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (comment_id, rating, fields=None))]
    /// Updates score of authenticated user for given comment. Valid scores are -1, 0 and 1.
    /// (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.rate_comment` for parameters and return type
    pub async fn rate_comment(
        &self,
        comment_id: u32,
        rating: i8,
        fields: Option<Vec<String>>,
    ) -> PyResult<CommentResource> {
        self.client
            .with_optional_fields(fields)
            .rate_comment(comment_id, rating)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (query=None, fields=None, limit=None, offset=None))]
    /// List the users currently registered on the site (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_users` for parameters and return type
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
    /// Creates a new user using specified parameters (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.create_user` for parameters and return type
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
    #[allow(clippy::too_many_arguments)]
    /// Updates an existing user using specified parameters (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.update` for parameters and return type
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
    /// Retrieves information about an existing user (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.get_user` for parameters and return type
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

    /// Deletes an existing user (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.delete_user` for parameters and return type
    pub async fn delete_user(&self, user_name: String, version: u32) -> PyResult<()> {
        self.client
            .request()
            .delete_user(user_name, version)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (user_name, fields=None))]
    /// Fetches a list of the given user's auth tokens (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_user_tokens` for parameters and return type
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

    #[pyo3(signature = (user_name, note=None, enabled=None, expiration_time=None, fields=None))]
    /// Creates an auth token for the given user (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.create_user_token` for parameters and return type
    pub async fn create_user_token(
        &self,
        user_name: String,
        note: Option<String>,
        enabled: Option<bool>,
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
        if let Some(enabled) = enabled {
            cutoken.enabled(enabled);
        }
        let cutoken = cutoken.build()?;
        self.client
            .with_optional_fields(fields)
            .create_user_token(user_name, &cutoken)
            .await
            .map_err(Into::into)
    }

    #[pyo3(signature = (user_name, token, version, enabled=None, note=None, expiration_time=None, fields=None))]
    #[allow(clippy::too_many_arguments)]
    /// Update a user's existing auth token (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.update_user_token` for parameters and return type
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

    /// Deletes an existing user auth token (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.delete_user_token` for parameters and return type
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

    /// Start a password reset request (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.password_reset_request` for parameters and return type
    pub async fn password_reset_request(&self, email_or_name: String) -> PyResult<()> {
        self.client
            .request()
            .password_reset_request(email_or_name)
            .await
            .map_err(Into::into)
    }

    /// Confirm a password reset request (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.password_reset_confirm` for parameters and return type
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
    /// List the snapshots currently available on the site (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.list_snapshots` for parameters and return type
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

    /// Retrieves simple statistics. ``featured_post`` is ``None`` if there is no featured post yet.
    /// ``server_time`` is pretty much the same as the Date HTTP
    /// field, only formatted in a manner consistent with other dates. Values in config key are
    /// taken directly from the server config, with the exception of privilege array keys being
    /// converted to lower camel case to match the API convention.
    ///
    /// (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.global_info` for parameters and return type
    pub async fn global_info(&self) -> PyResult<GlobalInfo> {
        self.client
            .request()
            .get_global_info()
            .await
            .map_err(Into::into)
    }

    /// Puts a file from a given file path in temporary storage and assigns it a token that can be
    /// used in other requests. (async version)
    ///
    /// :see: :func:`~szurubooru_client.SzurubooruSyncClient.upload_temporary_file` for parameters and return type
    pub async fn upload_temporary_file(&self, file_path: PathBuf) -> PyResult<String> {
        self.client
            .request()
            .upload_temporary_file_from_path(file_path)
            .await
            .map_err(Into::into)
            .map(|t| t.token)
    }
}
