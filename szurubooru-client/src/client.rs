#![warn(missing_docs)]

use crate::models::WithBaseURL;
use crate::{errors::*, models::*, tokens::*};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures_util::TryStreamExt;
use reqwest::header::CONTENT_TYPE;
use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION},
    multipart::{Form, Part},
    Client, ClientBuilder, Method, RequestBuilder, Response,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::fmt::{Display, Formatter};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fs::File, io::Read};
use url::Url;

///
/// The base Szurubooru Client
///
/// Use this `struct` to create requests to run against a Szurubooru instance.
///
#[derive(Debug)]
pub struct SzurubooruClient {
    base_url: Url,
    client: Client,
    auth: SzurubooruAuth,
}

impl SzurubooruClient {
    ///
    /// Construct a new `SzurubooruClient` using a username and token.
    ///
    /// * `host` - The host to connect to, including `http` or `https`. Any trailing slashes will
    ///             be stripped
    /// * `username` - The username to authenticate as
    /// * `token` - The token used to authenticate as `username`
    /// * `allow_insecure` - Whether to disable SSL verification
    ///
    /// ## Returns
    ///
    /// A [SzurubooruResult] containing the client. May return a [SzurubooruClientError::UrlParseError]
    /// if the host URL isn't a proper URL.
    ///
    /// ```no_run
    /// use szurubooru_client::SzurubooruClient;
    /// let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
    /// ```
    pub fn new_with_token(
        host: &str,
        username: &str,
        token: &str,
        allow_insecure: bool,
    ) -> SzurubooruResult<Self> {
        let encoded_auth = STANDARD.encode(format!("{username}:{token}").as_bytes());
        let token_header_value = format!("Token {encoded_auth}");
        let auth = SzurubooruAuth::TokenAuth(token_header_value);
        SzurubooruClient::new(host, auth, allow_insecure)
    }

    ///
    /// Construct a new `SzurubooruClient` using a username and token.
    ///
    /// * `host` - The host to connect to, including `http` or `https`
    /// * `username` - The username to authenticate as
    /// * `password` - The password used to authenticate as `username`
    /// * `allow_insecure` - Whether to disable SSL verification
    ///
    /// ## Returns
    ///
    /// A [SzurubooruResult] containing the client. May return a [SzurubooruClientError::UrlParseError]
    /// if the host URL isn't a proper URL.
    ///
    /// ```no_run
    /// use szurubooru_client::SzurubooruClient;
    /// let client = SzurubooruClient::new_with_basic_auth("http://localhost:5001", "myuser",
    ///     "mypassword", true).unwrap();
    /// ```
    pub fn new_with_basic_auth(
        host: &str,
        username: &str,
        password: &str,
        allow_insecure: bool,
    ) -> SzurubooruResult<Self> {
        let auth = SzurubooruAuth::BasicAuth(username.to_string(), password.to_string());
        SzurubooruClient::new(host, auth, allow_insecure)
    }

    /// Create a new client with anonymous credentials
    pub fn new_anonymous(host: &str, allow_insecure: bool) -> SzurubooruResult<Self> {
        let auth = SzurubooruAuth::None;
        SzurubooruClient::new(host, auth, allow_insecure)
    }

    fn new(host: &str, auth: SzurubooruAuth, allow_insecure: bool) -> SzurubooruResult<Self> {
        let host = if host.ends_with("/") {
            &host[0..host.len() - 1]
        } else {
            host
        };
        let mut base_url = Url::parse(host).map_err(|e| SzurubooruClientError::UrlParseError {
            source: e,
            url: host.to_string(),
        })?;
        base_url.set_fragment(None);

        let mut header_map = HeaderMap::new();
        //header_map.append(AUTHORIZATION, token_header_value.parse().unwrap());
        header_map.append(ACCEPT, "application/json".parse().unwrap());
        header_map.append(CONTENT_TYPE, "application/json".parse().unwrap());

        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(allow_insecure)
            .default_headers(header_map)
            .build()
            .unwrap();

        Ok(Self {
            base_url,
            client,
            auth,
        })
    }

    /// Construct a new request using the existing client auth and base URL
    /// All requests start with the [SzurubooruClient] struct.
    /// The [request](crate::SzurubooruClient::request),
    /// [with_fields](crate::SzurubooruClient::with_fields),
    /// [with_limit](crate::SzurubooruClient::with_limit) and
    /// [with_offset](crate::SzurubooruClient::with_offset) methods all return a [SzurubooruRequest] struct that will
    /// enable you to actually make the requests.
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # #[allow(unused)]
    /// # async {
    /// let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
    /// let new_request = client.request();
    /// let tag_categories = new_request.list_tag_categories().await;
    /// # };
    /// # ()
    /// ```
    pub fn request(&self) -> SzurubooruRequest {
        SzurubooruRequest::new(self)
    }

    /// Construct a new request while selecting only the given fields
    /// The Szurubooru API supports selecting a subset of fields for a given resource.
    /// Most resource [models](crate::models) have [Option] fields because of that.
    /// The default is to return all fields for a given resource.
    /// See [here](https://github.com/rr-/szurubooru/blob/master/doc/API.md#field-selecting) for
    /// more details
    ///
    /// For example, to select only the `version`, `id` and `content_url` fields of a
    /// [PostResource]
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # #[allow(unused)]
    /// # async {
    /// let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
    /// let new_request = client.request().with_fields(vec!["version".to_string(), "id".to_string(), "content_url".to_string()]);
    /// # };
    /// # ()
    /// ```
    pub fn with_fields(&self, fields: Vec<String>) -> SzurubooruRequest {
        self.request().with_fields(fields)
    }

    /// The same as [with_fields](SzurubooruClient::with_fields), but accepts an Option type instead
    pub fn with_optional_fields(&self, fields: Option<Vec<String>>) -> SzurubooruRequest {
        self.request().with_optional_fields(fields)
    }

    /// Construct a new request with the given limit
    /// The Szurubooru API supports limiting the number of resources returned for Paginated
    /// API endpoints.
    ///
    /// For example, to limit the number of pools returned by [list_pools](SzurubooruRequest::list_pools)
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # #[allow(unused)]
    /// # async {
    /// let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
    /// // Limit the number of results per page to 10
    /// let pools_result = client.with_limit(10)
    ///                         .list_pools(None)
    ///                         .await;
    /// # };
    /// # ()
    /// ```
    pub fn with_limit(&self, limit: u32) -> SzurubooruRequest {
        self.request().with_limit(limit)
    }

    /// The same as [with_limit](SzurubooruClient::with_limit), but accepts an Option type instead
    pub fn with_optional_limit(&self, limit: Option<u32>) -> SzurubooruRequest {
        self.request().with_optional_limit(limit)
    }

    /// Construct a new request starting at the given offset
    /// The Szurubooru API supports offsetting the results returned from Paginated API
    /// endpoints. Use this offset in combination with the limit to page through
    /// large result sets.
    ///
    /// For example, to offset the list of pools returned by [list_pools](SzurubooruRequest::list_pools)
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # #[allow(unused)]
    /// # async {
    /// let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
    /// // Skip the first ten pools in the list
    /// let pools_result = client.with_offset(10)
    ///                         .list_pools(None)
    ///                         .await;
    /// # };
    /// # ()
    /// ```
    pub fn with_offset(&self, offset: u32) -> SzurubooruRequest {
        self.request().with_offset(offset)
    }

    /// The same as [with_offset](SzurubooruClient::with_offset), but accepts an Option type instead
    pub fn with_optional_offset(&self, offset: Option<u32>) -> SzurubooruRequest {
        self.request().with_optional_offset(offset)
    }
}

#[derive(Debug)]
/// A type that represents a single Szurubooru request.
pub struct SzurubooruRequest<'a> {
    /// The currently selected fields to return (if applicable)
    pub fields: Option<Vec<String>>,
    /// The maximum number of resources to return (if supported by the API endpoint)
    pub limit: Option<u32>,
    /// The number of resource to skip before returning any results
    /// (if supported by the API endpoint)
    pub offset: Option<u32>,
    client: &'a SzurubooruClient,
}

impl<'a> SzurubooruRequest<'a> {
    pub(super) fn new(client: &'a SzurubooruClient) -> Self {
        Self {
            client,
            fields: None,
            limit: None,
            offset: None,
        }
    }

    /// Select which fields to return from the query.
    /// The Szurubooru API supports selecting a subset of fields for a given resource.
    /// Most resource [models](crate::models) have [Option] fields because of that.
    /// The default is to return all fields for a given resource.
    /// See [here](https://github.com/rr-/szurubooru/blob/master/doc/API.md#field-selecting) for
    /// more details
    ///
    /// For example, to select only the `version`, `id` and `content_url` fields of a
    /// [PostResource]
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # #[allow(unused)]
    /// # async {
    /// let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
    /// let new_request = client.request().with_fields(vec!["version".to_string(), "id".to_string(), "content_url".to_string()]);
    /// # };
    /// # ()
    /// ```
    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// The same as [with_fields](SzurubooruRequest::with_fields), but accepts an Option type instead
    pub fn with_optional_fields(self, val: Option<Vec<String>>) -> Self {
        match val {
            Some(f) => self.with_fields(f),
            None => self,
        }
    }

    /// Limit the number of returned results
    /// The Szurubooru API supports limiting the number of resources returned for Paginated
    /// API endpoints.
    ///
    /// For example, to limit the number of pools returned by [list_pools](SzurubooruRequest::list_pools)
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # #[allow(unused)]
    /// # async {
    /// let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
    /// // Limit the number of results per page to 10
    /// let pools_result = client.with_limit(10)
    ///                         .list_pools(None)
    ///                         .await;
    /// # };
    /// # ()
    /// ```
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// The same as [with_limit](SzurubooruRequest::with_limit), but accepts an Option type instead
    pub fn with_optional_limit(self, val: Option<u32>) -> Self {
        match val {
            Some(f) => self.with_limit(f),
            None => self,
        }
    }

    /// Skip a certain number of records
    /// The Szurubooru API supports offsetting the results returned from Paginated API
    /// endpoints. Use this offset in combination with the limit to page through
    /// large result sets.
    ///
    /// For example, to offset the list of pools returned by [list_pools](SzurubooruRequest::list_pools)
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # #[allow(unused)]
    /// # async {
    /// let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
    /// // Skip the first ten pools in the list
    /// let pools_result = client.with_offset(10)
    ///                         .list_pools(None)
    ///                         .await;
    /// # };
    /// # ()
    /// ```
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// The same as [with_offset](SzurubooruRequest::with_offset), but accepts an Option type instead
    pub fn with_optional_offset(self, val: Option<u32>) -> Self {
        match val {
            Some(f) => self.with_offset(f),
            None => self,
        }
    }

    #[doc(hidden)]
    fn prep_request<T>(
        &self,
        method: Method,
        path: T,
        query: Option<&Vec<QueryToken>>,
    ) -> reqwest::RequestBuilder
    where
        T: AsRef<str> + Display,
    {
        let mut req_url = if !path.as_ref().contains(&self.client.base_url.to_string()) {
            let mut url = self.client.base_url.clone();
            url.set_path(path.as_ref());
            url
        } else {
            Url::parse(path.as_ref()).unwrap()
        };

        if let Some(query_vec) = query {
            let mut qpm = req_url.query_pairs_mut();
            let query_string = query_vec.to_query_string();
            qpm.append_pair("query", &query_string);
        }

        if let Some(fields) = &self.fields {
            let mut qpm = req_url.query_pairs_mut();
            let fields_list = fields.join(",");
            qpm.append_pair("fields", &fields_list);
        }

        if let Some(limit) = &self.limit {
            let mut qpm = req_url.query_pairs_mut();
            qpm.append_pair("limit", &limit.to_string());
        }

        if let Some(offset) = &self.offset {
            let mut qpm = req_url.query_pairs_mut();
            qpm.append_pair("offset", &offset.to_string());
        }

        // This doesn't detect the required `mut` for some reason
        #[allow(unused_mut)]
        let mut req = self.client.client.request(method, req_url);
        match &self.client.auth {
            SzurubooruAuth::TokenAuth(t) => {
                let mut header_map = HeaderMap::new();
                header_map.append(AUTHORIZATION, t.parse().unwrap());

                req.headers(header_map)
            }
            SzurubooruAuth::BasicAuth(u, p) => req.basic_auth(u, Some(p)),
            SzurubooruAuth::None => req,
        }
    }

    #[tracing::instrument(skip(self), fields(base_url=self.client.base_url.to_string()))]
    async fn do_request<T, B, P>(
        &self,
        method: Method,
        path: P,
        query: Option<&Vec<QueryToken>>,
        body: Option<&B>,
    ) -> SzurubooruResult<T>
    where
        T: DeserializeOwned,
        B: Serialize + std::fmt::Debug,
        P: AsRef<str> + Display + std::fmt::Debug,
    {
        let mut request = self.prep_request(method, path, query);

        if let Some(b) = body {
            let b_str =
                serde_json::to_string(b).map_err(SzurubooruClientError::JSONSerializationError)?;
            request = request.body(b_str);
        }

        self.handle_request(request).await
    }

    async fn handle_response(&self, response: Response) -> SzurubooruResult<Response> {
        if response.status().is_client_error() || response.status().is_server_error() {
            let status = response.status();
            let resp_json = response
                .text()
                .await
                .map_err(SzurubooruClientError::RequestError)?;

            let server_error = serde_json::from_str::<SzurubooruServerError>(&resp_json)
                .map_err(|_e| SzurubooruClientError::ResponseError(status, resp_json))?;
            Err(SzurubooruClientError::SzurubooruServerError(server_error))
        } else {
            Ok(response)
        }
    }

    async fn handle_request<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> SzurubooruResult<T> {
        let request = request
            .build()
            .map_err(SzurubooruClientError::RequestBuilderError)?;

        let response = self.client.client.execute(request).await;

        let response = self
            .handle_response(response.map_err(SzurubooruClientError::RequestError)?)
            .await?;

        let response_text = response
            .text()
            .await
            .map_err(SzurubooruClientError::RequestError)?;

        serde_json::from_str::<SzuruEither<T, SzurubooruServerError>>(&response_text)
            .map_err(|e| SzurubooruClientError::ResponseParsingError(e, response_text))?
            .into_result()
    }

    fn propagate_urls<T>(&self, wbu: T) -> T
    where
        T: WithBaseURL,
    {
        #[allow(clippy::unnecessary_to_owned)]
        wbu.with_base_url(&self.client.base_url.to_string())
    }

    /// Lists all tag categories. Doesn't use paging.
    pub async fn list_tag_categories(
        &self,
    ) -> SzurubooruResult<UnpagedSearchResult<TagCategoryResource>> {
        self.do_request(Method::GET, "/api/tag-categories", None, None::<&String>)
            .await
    }

    /// Creates a new tag category using specified parameters. Name must match
    /// `tag_category_name_regex` from server's configuration. First category created
    /// becomes the default category.
    pub async fn create_tag_category(
        &self,
        new_cat: &CreateUpdateTagCategory,
    ) -> SzurubooruResult<TagCategoryResource> {
        self.do_request(Method::POST, "/api/tag-categories", None, Some(new_cat))
            .await
    }

    /// Updates an existing tag category using specified parameters. Name must match
    /// `tag_category_name_regex` from server's configuration. All fields except
    /// [version](crate::models::TagCategoryResource::version) are optional - update concerns only provided fields.
    pub async fn update_tag_category<T>(
        &self,
        name: T,
        update_tag_cat: &CreateUpdateTagCategory,
    ) -> SzurubooruResult<TagCategoryResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/tag-category/{name}");
        self.do_request(Method::PUT, &path, None, Some(update_tag_cat))
            .await
    }

    /// Retrieves information about an existing tag category.
    pub async fn get_tag_category<T>(&self, name: T) -> SzurubooruResult<TagCategoryResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/tag-category/{name}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
    }

    /// Deletes existing tag category. The tag category to be deleted must have no usages.
    pub async fn delete_tag_category<T>(&self, name: T, version: u32) -> SzurubooruResult<()>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/tag-category/{name}");
        let version_obj = ResourceVersion { version };
        self.do_request::<Value, _, _>(Method::DELETE, &path, None, Some(&version_obj))
            .await
            .map(|_| ())
    }

    /// Sets given tag category as default. All new tags created manually or automatically will
    /// have this category.
    pub async fn set_default_tag_category<T>(&self, name: T) -> SzurubooruResult<()>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/tag-category/{name}/default");
        self.do_request(Method::PUT, &path, None, None::<&String>)
            .await
    }

    /// Searches for tags.
    /// See the [named tokens](crate::tokens::TagNamedToken) and
    /// [sort tokens](crate::tokens::TagSortToken) for all possible query tokens, or use
    /// [QueryToken] for a custom token
    pub async fn list_tags(
        &self,
        query: Option<&Vec<QueryToken>>,
    ) -> SzurubooruResult<PagedSearchResult<TagResource>> {
        self.do_request(Method::GET, "/api/tags", query, None::<&String>)
            .await
    }

    /// Creates a new tag using specified parameters. Names, suggestions and implications must
    /// match `tag_name_regex` from server's configuration. Category must exist and is the same
    /// as the `name` field within [TagCategoryResource] resource.
    /// Suggestions and implications are optional. If specified implied tags or suggested tags do
    /// not exist yet, they will be automatically created. Tags created automatically have no
    /// implications, no suggestions, one name and their category is set to the first tag category
    /// found. If there are no tag categories established yet, an error will be thrown.
    pub async fn create_tag(&self, new_tag: &CreateUpdateTag) -> SzurubooruResult<TagResource> {
        self.do_request(Method::POST, "/api/tags", None, Some(new_tag))
            .await
    }

    /// Updates an existing tag using specified parameters. Names, suggestions and implications must
    /// match `tag_name_regex` from server's configuration. Category must exist and is the same
    /// as the `name` field within [TagCategoryResource] resource.
    /// Suggestions and implications are optional. If specified implied tags or suggested tags do
    /// not exist yet, they will be automatically created. Tags created automatically have no
    /// implications, no suggestions, one name and their category is set to the first tag category
    /// found. If there are no tag categories established yet, an error will be thrown.
    pub async fn update_tag<T>(
        &self,
        name: T,
        update_tag: &CreateUpdateTag,
    ) -> SzurubooruResult<TagResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/tag/{name}");
        self.do_request(Method::PUT, &path, None, Some(update_tag))
            .await
    }

    /// Retrieves information about an existing tag.
    pub async fn get_tag<T>(&self, name: T) -> SzurubooruResult<TagResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/tag/{name}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
    }

    /// Deletes existing tag. The tag to be deleted must have no usages.
    pub async fn delete_tag<T>(&self, name: T, version: u32) -> SzurubooruResult<()>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/tag/{name}");
        let version_obj = ResourceVersion { version };
        self.do_request::<Value, _, _>(Method::DELETE, &path, None, Some(&version_obj))
            .await
            .map(|_| ())
    }

    /// Removes source tag and merges all of its usages, suggestions and implications to the
    /// target tag. Other tag properties such as category and aliases do not get transferred
    /// and are discarded.
    pub async fn merge_tags(&self, merge_opts: &MergeTags) -> SzurubooruResult<TagResource> {
        self.do_request(Method::POST, "/api/tag-merge", None, Some(merge_opts))
            .await
    }

    /// Lists siblings of given tag, e.g. tags that were used in the same posts as the given tag.
    /// The [occurrences](crate::models::TagSibling::occurrences) field signifies how many times a given
    /// sibling appears with given tag. Results are sorted by occurrences count and the list is
    /// truncated to the first 50 elements. Doesn't use paging.
    pub async fn get_tag_siblings<T>(
        &self,
        name: T,
    ) -> SzurubooruResult<UnpagedSearchResult<TagSibling>>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/tag-siblings/{name}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
    }

    /// Searches for posts.
    /// See [PostNamedToken], [PostSortToken] and [PostSpecialToken] for valid tokens to use with
    /// this method, or use [QueryToken] to construct a custom token
    pub async fn list_posts(
        &self,
        query: Option<&Vec<QueryToken>>,
    ) -> SzurubooruResult<PagedSearchResult<PostResource>> {
        self.do_request(Method::GET, "/api/posts", query, None::<&String>)
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    async fn create_update_post_from_url(
        &self,
        path: &str,
        method: Method,
        cupost: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        if method == Method::POST && cupost.safety.is_none() {
            return Err(SzurubooruClientError::ValidationError(
                "Safety must be set".to_string(),
            ));
        }
        self.do_request(method, path, None, Some(cupost)).await
    }

    /// Create a new post based on the `contentUrl` field, which the server will use to download
    /// the image.
    /// If specified tags do not exist yet, they will be automatically created. Tags created
    /// automatically have no implications, no suggestions, one name and their category is set to
    /// the first tag category found. [safety](crate::models::CreateUpdatePost::safety) must be any of
    /// `safe`, `sketchy` or `unsafe`.
    /// Relations must contain valid post IDs. If `flag` is omitted, they will be defined by
    /// default (`"loop"` will be set for all video posts, and `"sound"` will be auto-detected).
    /// Sending empty thumbnail will cause the post to use default thumbnail. If `anonymous` is set
    /// to `true`, the uploader name won't be recorded (privilege verification still applies;
    /// it's possible to disallow anonymous uploads completely from config.)
    pub async fn create_post_from_url(
        &self,
        new_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        self.create_update_post_from_url("/api/posts", Method::POST, new_post)
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    /// Update an existing post
    /// See [SzurubooruRequest::create_post_from_url] for more details about the fields in
    /// [CreateUpdatePost]
    pub async fn update_post(
        &self,
        post_id: u32,
        update_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        let path = format!("/api/post/{post_id}");
        self.create_update_post_from_url(&path, Method::PUT, update_post)
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    /// Update an existing post from a given URL
    /// See [SzurubooruRequest::create_post_from_url] for more details about the fields in
    /// [CreateUpdatePost]
    pub async fn update_post_from_url(
        &self,
        post_id: u32,
        update_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        assert!(update_post.content_url.is_some());
        let path = format!("/api/post/{post_id}");
        self.create_update_post_from_url(&path, Method::PUT, update_post)
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    // Create function to upload by byte array in the future

    fn part_from_file(&self, file: &mut File) -> SzurubooruResult<Part> {
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)
            .map_err(SzurubooruClientError::IOError)?;

        Ok(Part::stream(bytes))
    }

    async fn create_update_post_from_file<T>(
        &self,
        file: Option<&mut File>,
        thumbnail: Option<&mut File>,
        file_name: Option<T>,
        path: &str,
        method: Method,
        cupost: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource>
    where
        T: AsRef<str>,
    {
        let request = self.prep_request(method, path, None);

        let metadata_str =
            serde_json::to_string(cupost).map_err(SzurubooruClientError::JSONSerializationError)?;
        let metadata_part = Part::text(metadata_str);

        let mut form = Form::new().part("metadata", metadata_part);

        if let Some(file) = file {
            let content_part = self
                .part_from_file(file)?
                .file_name(file_name.as_ref().unwrap().as_ref().to_string());
            form = form.part("content", content_part);
        }

        if let Some(thumbnail) = thumbnail {
            let thumbnail_part = self
                .part_from_file(thumbnail)?
                .file_name(format!("thumbnail_{}", file_name.unwrap().as_ref()));
            form = form.part("thumbnail", thumbnail_part);
        }

        self.handle_request(request.multipart(form)).await
    }

    /// Create a new post from a file handle
    /// See [SzurubooruRequest::create_post_from_url] for more details about the fields in
    /// [CreateUpdatePost]
    pub async fn create_post_from_file<T>(
        &self,
        file: &mut File,
        thumbnail: Option<&mut File>,
        file_name: T,
        new_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource>
    where
        T: AsRef<str>,
    {
        self.create_update_post_from_file(
            Some(file),
            thumbnail,
            Some(file_name),
            "/api/posts",
            Method::POST,
            new_post,
        )
        .await
        .map(|pr| self.propagate_urls(pr))
    }

    /// Create a new post from a file path
    /// See [SzurubooruRequest::create_post_from_url] for more details about the fields in
    /// [CreateUpdatePost]
    pub async fn create_post_from_file_path(
        &self,
        file_path: impl AsRef<Path>,
        thumbnail: Option<impl AsRef<Path>>,
        new_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        let mut file = File::open(&file_path).map_err(SzurubooruClientError::IOError)?;
        let filename = file_path.as_ref().file_name().unwrap().to_str().unwrap();
        let mut thumbnail_file = if let Some(t) = thumbnail {
            Some(File::open(t).map_err(SzurubooruClientError::IOError)?)
        } else {
            None
        };
        self.create_post_from_file(&mut file, thumbnail_file.as_mut(), filename, new_post)
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    /// Create a post from a token previously generated by
    /// [upload_temporary_file_from_path](SzurubooruRequest::upload_temporary_file_from_path)
    pub async fn create_post_from_token(
        &self,
        new_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        assert!(new_post.content_token.is_some());

        self.create_update_post_from_file(
            None,
            None,
            None::<String>,
            "/api/posts",
            Method::POST,
            new_post,
        )
        .await
        .map(|pr| self.propagate_urls(pr))
    }

    /// Update an existing post from an open File handle
    /// See [SzurubooruRequest::create_post_from_url] for more details about the fields in
    /// [CreateUpdatePost]
    pub async fn update_post_from_file(
        &self,
        post_id: u32,
        file: Option<&mut File>,
        thumbnail: Option<&mut File>,
        file_name: impl AsRef<str>,
        update_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        let path = format!("/api/post/{post_id}");
        self.create_update_post_from_file(
            file,
            thumbnail,
            Some(file_name),
            &path,
            Method::PUT,
            update_post,
        )
        .await
        .map(|pr| self.propagate_urls(pr))
    }

    /// Update an existing post from a file path
    /// See [SzurubooruRequest::create_post_from_url] for more details about the fields in
    /// [CreateUpdatePost]
    pub async fn update_post_from_file_path(
        &self,
        post_id: u32,
        file_path: Option<impl AsRef<Path>>,
        thumbnail: Option<impl AsRef<Path>>,
        update_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        let mut filename = None;
        let mut file = if let Some(f) = file_path {
            filename = Some(
                f.as_ref()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
            Some(File::open(f).map_err(SzurubooruClientError::IOError)?)
        } else {
            None
        };

        let mut thumbnail_file = if let Some(t) = thumbnail {
            if filename.is_none() {
                filename = Some(
                    t.as_ref()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }
            Some(File::open(t).map_err(SzurubooruClientError::IOError)?)
        } else {
            None
        };
        self.update_post_from_file(
            post_id,
            file.as_mut(),
            thumbnail_file.as_mut(),
            filename.unwrap(),
            update_post,
        )
        .await
        .map(|pr| self.propagate_urls(pr))
    }

    /// Update a post from a token previously generated by
    /// [upload_temporary_file_from_path](SzurubooruRequest::upload_temporary_file_from_path)
    pub async fn update_post_from_token(
        &self,
        post_id: u32,
        update_post: &CreateUpdatePost,
    ) -> SzurubooruResult<PostResource> {
        assert!(update_post.content_token.is_some());
        let url = format!("/api/post/{post_id}");
        self.create_update_post_from_file(
            None,
            None,
            None::<String>,
            &url,
            Method::PUT,
            update_post,
        )
        .await
        .map(|pr| self.propagate_urls(pr))
    }

    async fn get_post_content(
        &self,
        post_id: u32,
        get_thumbnail: bool,
    ) -> SzurubooruResult<Response> {
        let post_resource = self.get_post(post_id).await?;

        let content_path = if get_thumbnail {
            post_resource.thumbnail_url.unwrap()
        } else {
            post_resource.content_url.unwrap()
        };

        let req = self.prep_request(Method::GET, content_path, None);
        let request = req
            .build()
            .map_err(SzurubooruClientError::RequestBuilderError)?;

        let resp_res = self
            .client
            .client
            .execute(request)
            .await
            .map_err(SzurubooruClientError::RequestError)?;
        self.handle_response(resp_res).await
    }

    ///Fetches the given post ID's image as a stream of bytes
    pub async fn get_image_bytestream(
        &self,
        post_id: u32,
    ) -> SzurubooruResult<impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>>>
    {
        self.get_post_content(post_id, false)
            .await
            .map(|cr| cr.bytes_stream())
    }

    ///Fetches the given post ID's thumbnail as a stream of bytes
    pub async fn get_thumbnail_bytestream(
        &self,
        post_id: u32,
    ) -> SzurubooruResult<impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>>>
    {
        self.get_post_content(post_id, true)
            .await
            .map(|cr| cr.bytes_stream())
    }

    ///Fetches the given post ID's image as a [Bytes](bytes::Bytes) struct
    pub async fn get_image_bytes(&self, post_id: u32) -> SzurubooruResult<bytes::Bytes> {
        let content_response = self.get_post_content(post_id, false).await?;

        content_response
            .bytes()
            .await
            .map_err(SzurubooruClientError::RequestError)
    }

    ///Fetches the given post ID's thumbnail as a [Bytes](bytes::Bytes) struct
    pub async fn get_thumbnail_bytes(&self, post_id: u32) -> SzurubooruResult<bytes::Bytes> {
        let content_response = self.get_post_content(post_id, true).await?;

        content_response
            .bytes()
            .await
            .map_err(SzurubooruClientError::RequestError)
    }

    async fn write_content_to_file<S>(
        &self,
        file: &mut File,
        stream: &mut S,
    ) -> SzurubooruResult<()>
    where
        S: futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    {
        let mut writer = BufWriter::new(file);

        while let Some(bytes) = stream
            .try_next()
            .await
            .map_err(SzurubooruClientError::RequestError)?
        {
            writer
                .write_all(bytes.as_ref())
                .map_err(SzurubooruClientError::IOError)?;
        }

        Ok(())
    }

    ///Downloads a post's image and writes it to the given file handle
    pub async fn download_image_to_file(
        &self,
        post_id: u32,
        file: &mut File,
    ) -> SzurubooruResult<()> {
        let mut stream = self.get_image_bytestream(post_id).await?;
        self.write_content_to_file(file, &mut stream).await
    }

    ///Downloads a post's image and writes it to the given path
    pub async fn download_image_to_path(
        &self,
        post_id: u32,
        path: impl AsRef<Path>,
    ) -> SzurubooruResult<()> {
        let mut stream = self.get_image_bytestream(post_id).await?;
        let mut file = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.as_ref())
            .map_err(SzurubooruClientError::IOError)?;
        self.write_content_to_file(&mut file, &mut stream).await
    }

    ///Downloads a post's thumbnail and writes it to the given file handle
    pub async fn download_thumbnail_to_file(
        &self,
        post_id: u32,
        file: &mut File,
    ) -> SzurubooruResult<()> {
        let mut stream = self.get_thumbnail_bytestream(post_id).await?;
        self.write_content_to_file(file, &mut stream).await
    }

    ///Downloads a post's thumbnail and writes it to the given path
    pub async fn download_thumbnail_to_path(
        &self,
        post_id: u32,
        path: impl AsRef<Path>,
    ) -> SzurubooruResult<()> {
        let mut stream = self.get_thumbnail_bytestream(post_id).await?;
        let mut file = File::open(path.as_ref()).map_err(SzurubooruClientError::IOError)?;
        self.write_content_to_file(&mut file, &mut stream).await
    }

    /// Retrieves posts that look like the input image
    pub async fn reverse_search_file(
        &self,
        file: &mut File,
        file_path: impl AsRef<str>,
    ) -> SzurubooruResult<ImageSearchResult> {
        let request = self.prep_request(Method::POST, "/api/posts/reverse-search", None);

        let image_part = self
            .part_from_file(file)?
            .file_name(file_path.as_ref().to_string());
        let form = Form::new().part("content", image_part);

        self.handle_request(request.multipart(form))
            .await
            .map(|isr| self.propagate_urls(isr))
    }

    /// Retrieves posts that look like the input image from the given file path
    pub async fn reverse_search_file_path(
        &self,
        file_path: impl AsRef<Path>,
    ) -> SzurubooruResult<ImageSearchResult> {
        let mut file = File::open(&file_path).map_err(SzurubooruClientError::IOError)?;
        let filename = file_path.as_ref().file_name().unwrap().to_str().unwrap();
        self.reverse_search_file(&mut file, filename)
            .await
            .map(|isr| self.propagate_urls(isr))
    }

    // Need to add a reverse search for bytes

    /// Searches for an exact match of a file based on the SHA1 checksum
    pub async fn post_for_file(
        &self,
        mut file: &mut File,
    ) -> SzurubooruResult<Option<PostResource>> {
        let mut hasher = Sha1::new();
        std::io::copy(&mut file, &mut hasher).map_err(SzurubooruClientError::IOError)?;
        let hash = hasher.finalize();
        let hex_string = hex::encode(hash);

        let qt = QueryToken::token(PostNamedToken::ContentChecksum, hex_string);
        let psr = self
            .list_posts(Some(&vec![qt]))
            .await
            .map(|psr| self.propagate_urls(psr))?;
        Ok(psr.results.first().cloned())
    }

    /// Searches for an exact match of a file path based on the SHA1 checksum
    pub async fn post_for_file_path(
        &self,
        file_path: impl AsRef<Path>,
    ) -> SzurubooruResult<Option<PostResource>> {
        let mut file = File::open(file_path).map_err(SzurubooruClientError::IOError)?;

        self.post_for_file(&mut file).await
    }

    /// Retrieves information about an existing post.
    pub async fn get_post(&self, post_id: u32) -> SzurubooruResult<PostResource> {
        let path = format!("/api/post/{post_id}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    /// Retrieves information about posts that are before or after an existing post.
    pub async fn get_around_post(&self, post_id: u32) -> SzurubooruResult<AroundPostResult> {
        let path = format!("/api/post/{post_id}/around");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
    }

    /// Deletes existing post. Related posts and tags are kept.
    pub async fn delete_post(&self, post_id: u32, version: u32) -> SzurubooruResult<()> {
        let path = format!("/api/post/{post_id}");
        let version_obj = ResourceVersion { version };
        self.do_request::<Value, _, _>(Method::DELETE, &path, None, Some(&version_obj))
            .await
            .map(|_| ())
    }

    ///
    /// Removes source post and merges all of its tags, relations, scores, favorites and comments to
    /// the target post. If [MergePost::replace_post_content] is set to `true`, content of the target post
    /// is replaced using the content of the source post; otherwise it remains unchanged. Source
    /// post properties such as its safety, source, whether to loop the video and other scalar
    /// values do not get transferred and are discarded.
    ///
    pub async fn merge_post(&self, merge_opts: &MergePost) -> SzurubooruResult<PostResource> {
        self.do_request(Method::POST, "/api/post-merge/", None, Some(merge_opts))
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    /// Updates score of authenticated user for given post. Valid scores are -1, 0 and 1.
    pub async fn rate_post(&self, post_id: u32, score: i8) -> SzurubooruResult<PostResource> {
        if !(-1..=1).contains(&score) {
            return Err(SzurubooruClientError::ValidationError(
                "Score must be -1, 0 or 1".to_string(),
            ));
        }
        let rating_obj = RateResource { score };
        let path = format!("/api/post/{post_id}/score");
        self.do_request(Method::PUT, &path, None, Some(&rating_obj))
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    /// Marks the post as favorite for authenticated user.
    pub async fn favorite_post(&self, post_id: u32) -> SzurubooruResult<PostResource> {
        let path = format!("/api/post/{post_id}/favorite");
        self.do_request(Method::POST, &path, None, None::<&String>)
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    /// Unmarks the post as favorite for authenticated user.
    pub async fn unfavorite_post(&self, post_id: u32) -> SzurubooruResult<PostResource> {
        let path = format!("/api/post/{post_id}/favorite");
        self.do_request(Method::DELETE, &path, None, None::<&String>)
            .await
            .map(|pr| self.propagate_urls(pr))
    }

    /// Retrieves the post that is currently featured on the main page in web client. If no post is
    /// featured, the result will be [Option::None]. Note that this method exists mostly for
    /// compatibility with setting featured post - most of the time, you'd want to use query global
    /// info which contains more information.
    pub async fn get_featured_post(&self) -> SzurubooruResult<Option<PostResource>> {
        self.do_request(Method::GET, "/api/featured-post", None, None::<&String>)
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Features a post on the main page
    pub async fn set_featured_post(&self, post_id: u32) -> SzurubooruResult<PostResource> {
        let id_object = PostId { id: post_id };
        self.do_request(Method::POST, "/api/featured-post", None, Some(&id_object))
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Lists all pool categories. Doesn't use paging.
    pub async fn list_pool_categories(
        &self,
    ) -> SzurubooruResult<UnpagedSearchResult<PoolCategoryResource>> {
        self.do_request(Method::GET, "/api/pool-categories", None, None::<&String>)
            .await
    }

    /// Creates a new pool category using specified parameters. Name must match
    /// `pool_category_name_regex` from server's configuration. First category created becomes
    /// the default category.
    pub async fn create_pool_category(
        &self,
        new_cat: &CreateUpdatePoolCategory,
    ) -> SzurubooruResult<PoolCategoryResource> {
        self.do_request(Method::POST, "/api/pool-categories", None, Some(new_cat))
            .await
    }

    /// Updates an existing tag category using specified parameters. Name must match
    /// `tag_category_name_regex` from server's configuration. All fields except the
    /// [version](crate::models::CreateUpdatePoolCategory::version) field are optional - update concerns
    /// only the provided fields.
    pub async fn update_pool_category<T>(
        &self,
        category_name: T,
        update_cat: &CreateUpdatePoolCategory,
    ) -> SzurubooruResult<PoolCategoryResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/pool-category/{category_name}");
        self.do_request(Method::PUT, &path, None, Some(update_cat))
            .await
    }

    /// Retrieves information about an existing pool category.
    pub async fn get_pool_category<T>(
        &self,
        category_name: T,
    ) -> SzurubooruResult<PoolCategoryResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/pool-category/{category_name}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
    }

    /// Deletes existing pool category. The pool category to be deleted must have no usages.
    pub async fn delete_pool_category<T>(
        &self,
        category_name: T,
        version: u32,
    ) -> SzurubooruResult<()>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/pool-category/{category_name}");
        let resource_obj = ResourceVersion { version };
        self.do_request::<Value, _, _>(Method::DELETE, &path, None, Some(&resource_obj))
            .await
            .map(|_| ())
    }

    /// Sets given pool category as default. All new pools created manually or automatically will
    /// have this category.
    pub async fn set_default_pool_category<T>(
        &self,
        category_name: T,
    ) -> SzurubooruResult<PoolCategoryResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/pool-category/{category_name}/default");
        self.do_request(Method::PUT, &path, None, None::<&String>)
            .await
    }

    /// Searches for pools.
    /// Anonymous tokens are the same as the [name](crate::tokens::PoolNamedToken::Name) token
    pub async fn list_pools(
        &self,
        query: Option<&Vec<QueryToken>>,
    ) -> SzurubooruResult<PagedSearchResult<PoolResource>> {
        self.do_request(Method::GET, "/api/pools", query, None::<&String>)
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Creates a new pool using specified parameters. Names, suggestions and implications must
    /// match `pool_name_regex` from server's configuration. Category must exist and is the same as
    /// [name](crate::models::PoolCategoryResource::name) field.
    /// [posts](crate::models::CreateUpdatePool::posts) is an optional list of integer post IDs. If the
    /// specified posts do not exist, an error will be thrown.
    pub async fn create_pool(
        &self,
        create_update_pool: &CreateUpdatePool,
    ) -> SzurubooruResult<PoolResource> {
        self.do_request(Method::POST, "/api/pool", None, Some(create_update_pool))
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Updates an existing pool using specified parameters. [names](crate::models::CreateUpdatePool::names),
    /// must match `pool_name_regex` from server's configuration.
    /// [category](crate::models::CreateUpdatePool::category) must exist and is the same as
    /// [name](crate::models::PoolCategoryResource::name) field. [posts](crate::models::CreateUpdatePool::posts)
    /// is an optional list of integer post IDs. If the specified posts do not exist yet, an error
    /// will be thrown. The full list of post IDs must be provided if they are being updated, and
    /// the previous list of posts will be replaced with the new one. All fields except
    /// [version](crate::models::CreateUpdatePool::version) are optional - update concerns only provided
    /// fields.
    pub async fn update_pool(
        &self,
        pool_id: u32,
        create_update_pool: &CreateUpdatePool,
    ) -> SzurubooruResult<PoolResource> {
        let path = format!("/api/pool/{pool_id}");
        self.do_request(Method::PUT, &path, None, Some(create_update_pool))
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Retrieves information about an existing pool.
    pub async fn get_pool(&self, pool_id: u32) -> SzurubooruResult<PoolResource> {
        let path = format!("/api/pool/{pool_id}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Deletes existing pool. All posts in the pool will only have their relation to the pool
    /// removed.
    pub async fn delete_pool(&self, pool_id: u32, version: u32) -> SzurubooruResult<()> {
        let path = format!("/api/pool/{pool_id}");
        let version_obj = ResourceVersion { version };
        self.do_request::<Value, _, _>(Method::DELETE, &path, None, Some(&version_obj))
            .await
            .map(|_| ())
    }

    /// Removes source pool and merges all of its posts with the target pool. Other pool properties
    /// such as category and aliases do not get transferred and are discarded.
    pub async fn merge_pools(&self, merge_pool: &MergePool) -> SzurubooruResult<PoolResource> {
        self.do_request(Method::POST, "/api/pool-merge", None, Some(merge_pool))
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Searches for comments.
    /// Anonymous tokens are the same as the [text](crate::tokens::CommentNamedToken::Text) token
    pub async fn list_comments(
        &self,
        query: Option<&Vec<QueryToken>>,
    ) -> SzurubooruResult<PagedSearchResult<CommentResource>> {
        self.do_request(Method::GET, "/api/comments", query, None::<&String>)
            .await
    }

    /// Creates a new comment under given post
    pub async fn create_comment(
        &self,
        new_comment: &CreateUpdateComment,
    ) -> SzurubooruResult<CommentResource> {
        self.do_request(Method::POST, "/api/comments", None, Some(new_comment))
            .await
    }

    /// Updates an existing comment text
    pub async fn update_comment(
        &self,
        comment_id: u32,
        update_comment: &CreateUpdateComment,
    ) -> SzurubooruResult<CommentResource> {
        let path = format!("/api/comment/{comment_id}");
        self.do_request(Method::PUT, &path, None, Some(update_comment))
            .await
    }

    /// Retrieves information about an existing comment
    pub async fn get_comment(&self, comment_id: u32) -> SzurubooruResult<CommentResource> {
        let path = format!("/api/comment/{comment_id}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
    }

    /// Deletes existing comment
    pub async fn delete_comment(&self, comment_id: u32, version: u32) -> SzurubooruResult<()> {
        let path = format!("/api/comment/{comment_id}");
        let version_obj = ResourceVersion { version };
        self.do_request::<Value, _, _>(Method::DELETE, &path, None, Some(&version_obj))
            .await
            .map(|_| ())
    }

    /// Updates score of authenticated user for given comment. Valid scores are -1, 0 and 1.
    pub async fn rate_comment(
        &self,
        comment_id: u32,
        score: i8,
    ) -> SzurubooruResult<CommentResource> {
        if !(-1..=1).contains(&score) {
            return Err(SzurubooruClientError::ValidationError(
                "Score must be -1, 0 or 1".to_string(),
            ));
        }
        let path = format!("/api/comment/{comment_id}/score");
        let rating = RateResource { score };
        self.do_request(Method::PUT, &path, None, Some(&rating))
            .await
    }

    /// Searches for users
    /// Anonymous tokens are the same as the [name](crate::tokens::UserNamedToken::Name) token
    /// See [UserNamedToken] and [UserSortToken] for type-safe tokens
    pub async fn list_users(
        &self,
        query: Option<&Vec<QueryToken>>,
    ) -> SzurubooruResult<PagedSearchResult<UserResource>> {
        self.do_request(Method::GET, "/api/users", query, None::<&String>)
            .await
            .map(|r| self.propagate_urls(r))
    }

    async fn create_update_user(
        &self,
        method: Method,
        path: &str,
        new_user: &CreateUpdateUser,
        file: Option<&mut File>,
        file_name: Option<impl AsRef<str>>,
    ) -> SzurubooruResult<UserResource> {
        match file {
            None => self.do_request(method, path, None, Some(new_user)).await,
            Some(file) => {
                let request = self.prep_request(method, path, None);

                let metadata_str = serde_json::to_string(&new_user)
                    .map_err(SzurubooruClientError::JSONSerializationError)?;
                let metadata_part = Part::text(metadata_str);

                let content_part = self
                    .part_from_file(file)?
                    .file_name(file_name.unwrap().as_ref().to_string());

                let form = Form::new()
                    .part("avatar", content_part)
                    .part("metadata", metadata_part);

                self.handle_request(request.multipart(form)).await
            }
        }
    }

    /// Creates a new user using specified parameters. Names and passwords must match
    /// `user_name_regex` and `password_regex` from server's configuration, respectively.
    /// Email address, rank and avatar fields are optional. Avatar style can be either
    /// [gravatar](crate::models::UserAvatarStyle::Gravatar) or [manual](crate::models::UserAvatarStyle::Manual).
    /// `manual` avatar style requires client to pass also the `avatar` file.
    /// If the rank is empty and the user happens to be the first user ever created,
    /// become an administrator, whereas subsequent users will be given the rank indicated by
    /// `default_rank` in the server's configuration.
    pub async fn create_user(&self, new_user: &CreateUpdateUser) -> SzurubooruResult<UserResource> {
        self.do_request(Method::POST, "/api/users", None, Some(new_user))
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Create a [UserResource] with the included Avatar file
    /// See [create_user](SzurubooruRequest::create_user) for other applicable fields and
    /// restrictions
    pub async fn create_user_with_avatar_file(
        &self,
        avatar: &mut File,
        file_name: impl AsRef<str>,
        new_user: &CreateUpdateUser,
    ) -> SzurubooruResult<UserResource> {
        self.create_update_user(
            Method::POST,
            "/api/users",
            new_user,
            Some(avatar),
            Some(file_name),
        )
        .await
        .map(|r| self.propagate_urls(r))
    }

    /// Create a [UserResource] with the included Avatar file path
    /// See [create_user](SzurubooruRequest::create_user) for other applicable fields and
    /// restrictions
    pub async fn create_user_with_avatar_path(
        &self,
        avatar_path: impl AsRef<Path>,
        new_user: &CreateUpdateUser,
    ) -> SzurubooruResult<UserResource> {
        let mut file = File::open(&avatar_path).map_err(SzurubooruClientError::IOError)?;
        let filename = avatar_path.as_ref().file_name().unwrap().to_str().unwrap();
        self.create_update_user(
            Method::POST,
            "/api/users",
            new_user,
            Some(&mut file),
            Some(filename),
        )
        .await
        .map(|r| self.propagate_urls(r))
    }

    /// Updates user using specified parameters. Names and passwords must match
    /// `user_name_regex` and `password_regex` from server's configuration, respectively.
    /// Email address, rank and avatar fields are optional. Avatar style can be either
    /// [gravatar](crate::models::UserAvatarStyle::Gravatar) or [manual](crate::models::UserAvatarStyle::Manual).
    /// `manual` avatar style requires client to pass also the `avatar` file.
    /// All fields except the [version](crate::models::CreateUpdateUser::version) are optional
    /// - update concerns only provided fields.
    pub async fn update_user<T>(
        &self,
        name: T,
        update_user: &CreateUpdateUser,
    ) -> SzurubooruResult<UserResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user/{name}");
        self.do_request(Method::PUT, path, None, Some(update_user))
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Update a [UserResource] with the included Avatar file
    /// See [update_user](SzurubooruRequest::update_user) for other applicable fields and
    /// restrictions
    pub async fn update_user_with_avatar_file<T>(
        &self,
        name: T,
        avatar: &mut File,
        file_name: impl AsRef<str>,
        update_user: &CreateUpdateUser,
    ) -> SzurubooruResult<UserResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user/{name}");
        self.create_update_user(
            Method::PUT,
            &path,
            update_user,
            Some(avatar),
            Some(file_name),
        )
        .await
        .map(|r| self.propagate_urls(r))
    }

    /// Update a [UserResource] with the included Avatar file path
    /// See [update_user](SzurubooruRequest::update_user) for other applicable fields and
    /// restrictions
    pub async fn update_user_with_avatar_path<T>(
        &self,
        name: T,
        avatar_path: impl AsRef<Path>,
        new_user: &CreateUpdateUser,
    ) -> SzurubooruResult<UserResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user/{name}");
        let mut file = File::open(&avatar_path).map_err(SzurubooruClientError::IOError)?;
        let filename = avatar_path.as_ref().file_name().unwrap().to_str().unwrap();
        self.create_update_user(
            Method::PUT,
            &path,
            new_user,
            Some(&mut file),
            Some(filename),
        )
        .await
        .map(|r| self.propagate_urls(r))
    }

    /// Retrieves information about an existing user
    pub async fn get_user<T>(&self, name: T) -> SzurubooruResult<UserResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user/{name}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Deletes existing user
    pub async fn delete_user<T>(&self, name: T, version: u32) -> SzurubooruResult<()>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user/{name}");
        let version_obj = ResourceVersion { version };
        self.do_request::<Value, _, _>(Method::DELETE, &path, None, Some(&version_obj))
            .await
            .map(|_| ())
    }

    /// Listing user tokens for the given user.
    pub async fn list_user_tokens<T>(
        &self,
        name: T,
    ) -> SzurubooruResult<UnpagedSearchResult<UserAuthTokenResource>>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user-tokens/{name}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Creates a new user token that can be used for authentication of API endpoints
    /// instead of a password.
    pub async fn create_user_token<T>(
        &self,
        user_name: T,
        create_token: &CreateUpdateUserAuthToken,
    ) -> SzurubooruResult<UserAuthTokenResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user-token/{user_name}");
        self.do_request(Method::POST, &path, None, Some(create_token))
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Updates an existing user token using specified parameters. All fields except the
    /// [version](crate::models::CreateUpdateUserAuthToken::version) are optional - update concerns only
    /// provided fields.
    pub async fn update_user_token<T>(
        &self,
        name: T,
        token: T,
        update_token: &CreateUpdateUserAuthToken,
    ) -> SzurubooruResult<UserAuthTokenResource>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user-token/{name}/{token}");
        self.do_request(Method::PUT, &path, None, Some(update_token))
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Deletes an existing user token using specified parameters. All fields except the
    /// [version](crate::models::CreateUpdateUserAuthToken::version) are optional - update concerns only
    /// provided fields.
    pub async fn delete_user_token<T>(
        &self,
        name: T,
        token: T,
        version: u32,
    ) -> SzurubooruResult<()>
    where
        T: AsRef<str> + Display,
    {
        let path = format!("/api/user-token/{name}/{token}");
        let version_obj = ResourceVersion { version };
        self.do_request::<Value, _, _>(Method::DELETE, &path, None, Some(&version_obj))
            .await
            .map(|_| ())
    }

    /// Sends a confirmation email to given user. The email contains link containing a token. The
    /// token cannot be guessed, thus using such link proves that the person who requested to reset
    /// the password also owns the mailbox, which is a strong indication they are the rightful
    /// owner of the account.
    /// Argument is either the user's username or email address
    pub async fn password_reset_request<T>(&self, email_or_name: T) -> SzurubooruResult<()>
    where
        T: AsRef<str> + Display,
    {
        let encoded = STANDARD.encode(email_or_name.as_ref().as_bytes());
        let path = format!("/api/password-reset/{encoded}");
        self.do_request(Method::GET, &path, None, None::<&String>)
            .await
    }

    /// Generates a new password for given user. Password is sent as plain-text, so it is
    /// recommended to connect through HTTPS.
    pub async fn password_reset_confirm<T>(
        &self,
        email_or_name: T,
        token: impl AsRef<str>,
    ) -> SzurubooruResult<TemporaryPassword>
    where
        T: AsRef<str> + Display,
    {
        let encoded = STANDARD.encode(email_or_name.as_ref().as_bytes());
        let path = format!("/api/password-reset/{encoded}");
        let token_obj = PasswordResetToken {
            token: token.as_ref().to_string(),
        };
        self.do_request(Method::POST, &path, None, Some(&token_obj))
            .await
    }

    /// Lists recent resource snapshots.
    /// See [SnapshotNamedToken] for query tokens.
    /// There are no sort tokens. The snapshots are always sorted by creation time.
    pub async fn list_snapshots(
        &self,
        query: Option<&Vec<QueryToken>>,
    ) -> SzurubooruResult<PagedSearchResult<SnapshotResource>> {
        self.do_request(Method::GET, "/api/snapshots", query, None::<&String>)
            .await
            .map(|r| self.propagate_urls(r))
    }

    /// Retrieves simple statistics. [featured_post](crate::models::GlobalInfo::featured_post) is
    /// [None] if there is no featured post yet.
    /// [server_time](crate::models::GlobalInfo::server_time) is pretty much the same as the Date HTTP
    /// field, only formatted in a manner consistent with other dates. Values in config key are
    /// taken directly from the server config, except for the privilege array keys being
    /// converted to lower camel case to match the API convention.
    pub async fn get_global_info(&self) -> SzurubooruResult<GlobalInfo> {
        self.do_request(Method::GET, "/api/info", None, None::<&String>)
            .await
    }

    /// Puts a file in temporary storage and assigns it a token that can be used in other requests.
    /// The files uploaded that way are deleted after a short while so clients shouldn't use it
    /// as a free upload service.
    pub async fn upload_temporary_file(
        &self,
        file: &mut File,
        file_name: impl AsRef<str>,
    ) -> SzurubooruResult<TemporaryFileUpload> {
        let request = self.prep_request(Method::POST, "/api/uploads", None);

        let content_part = self
            .part_from_file(file)?
            .file_name(file_name.as_ref().to_string());

        let form = Form::new().part("content", content_part);

        self.handle_request(request.multipart(form)).await
    }

    /// Puts a file from a given file path in temporary storage and assigns it a token that can be
    /// used in other requests.
    /// The files uploaded that way are deleted after a short while so clients shouldn't use it
    /// as a free upload service.
    pub async fn upload_temporary_file_from_path(
        &self,
        file_path: impl AsRef<Path>,
    ) -> SzurubooruResult<TemporaryFileUpload> {
        let mut file = File::open(&file_path).map_err(SzurubooruClientError::IOError)?;
        let filename = file_path.as_ref().file_name().unwrap().to_str().unwrap();

        self.upload_temporary_file(&mut file, filename).await
    }
}

/// Which kind of authentication is used. Automatically hides any sensitive information when printed
/// using [Debug](std::fmt::Debug)
enum SzurubooruAuth {
    // The encoded token
    TokenAuth(String),
    BasicAuth(String, String),
    #[allow(dead_code)]
    None,
}

impl std::fmt::Debug for SzurubooruAuth {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SzurubooruAuth ()")
    }
}
