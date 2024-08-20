use chrono::Months;
use sha1::{Digest, Sha1};
use std::error::Error;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use szurubooru_client::models::*;
use szurubooru_client::tokens::{CommentNamedToken, QueryToken};
use szurubooru_client::*;
use tempfile::tempfile;
use tokio::process::Command;
use tracing::level_filters::LevelFilter;
use tracing::{info, instrument};
use tracing_subscriber::filter::EnvFilter;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    let ev = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .compact()
        //.with_span_events(FmtSpan::ACTIVE)
        .with_target(false)
        .with_level(false)
        .with_env_filter(ev)
        .init();

    //info!("Starting Szurubooru instance...");
    let anon_client = start_instance().await;

    let create_user = CreateUpdateUserBuilder::default()
        .name("integration_user".to_string())
        .password("integration_password".to_string())
        .rank(UserRank::Administrator)
        .avatar_style(UserAvatarStyle::Gravatar)
        .build()
        .expect("Unable to create CreateUser object");
    let _user = anon_client
        .request()
        .create_user(&create_user)
        .await
        .expect("Error creating user");

    let auth_client = SzurubooruClient::new_with_basic_auth(
        "http://localhost:9801",
        "integration_user",
        "integration_password",
        true,
    )?;

    test_tag_categories(&auth_client).await;
    test_tags(&auth_client).await;
    test_creating_posts(&auth_client).await;
    test_pool_categories(&auth_client).await;
    test_pools(&auth_client).await;
    test_comments(&auth_client).await;
    test_users(&auth_client).await;
    test_snapshots(&auth_client).await;
    test_downloads(&auth_client).await;

    Command::new("sh")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("-c")
        .arg("./stop_szurubooru.sh")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .status()
        .await
        .expect("Failed to stop szurubooru");
    Ok(())
}

#[tracing::instrument]
async fn start_instance() -> SzurubooruClient {
    info!("Connecting to Szurubooru instance...");
    let anon_client = SzurubooruClient::new_anonymous("http://localhost:9801", true)
        .expect("Can't create anonymous client");

    let mut connected = false;
    let mut error = None;

    for _ in 0..5 {
        let info = anon_client.request().get_global_info().await;
        if info.is_ok() {
            connected = true;
            break;
        } else if let Err(e) = info {
            error = Some(e);
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    if !connected {
        panic!(
            "{}",
            format!("Unable to connect to instance. Last error: {error:?}")
        );
    }

    anon_client
}

#[tracing::instrument(skip(client), level = "INFO")]
async fn test_tag_categories(client: &SzurubooruClient) {
    info!("Testing tag categories");
    let tag_cats = client
        .request()
        .list_tag_categories()
        .await
        .expect("Could not list tag categories");
    assert_eq!(tag_cats.results.len(), 1);

    info!("Creating tag category");
    let create_tag_cat = CreateUpdateTagCategoryBuilder::default()
        .name("my_tag_cat".to_string())
        .color("purple".to_string())
        .order(1)
        .build()
        .expect("Unable to build create_tag_cat object");
    let result_tag_cat = client
        .request()
        .create_tag_category(&create_tag_cat)
        .await
        .expect("Unable to create tag category");
    assert_eq!(result_tag_cat.name, Some("my_tag_cat".to_string()));
    let tag_cats = client
        .request()
        .list_tag_categories()
        .await
        .expect("Could not list tag categories again");
    assert_eq!(tag_cats.results.len(), 2);

    info!("GETting tag category");
    let tag_res = client
        .request()
        .get_tag_category("my_tag_cat")
        .await
        .expect("Could not fetch tag category");
    assert_eq!(result_tag_cat.color, tag_res.color);

    info!("Updating tag category");
    let update_tag_cat = CreateUpdateTagCategoryBuilder::default()
        .version(tag_res.version)
        .color("red".to_string())
        .build()
        .expect("Could not create update for tag category");
    let update_res = client
        .request()
        .update_tag_category("my_tag_cat", &update_tag_cat)
        .await
        .expect("Unable to update tag category");
    assert_eq!(update_res.color, Some("red".to_string()));

    info!("Deleting tag category");
    client
        .request()
        .delete_tag_category("my_tag_cat", update_res.version)
        .await
        .expect("Could not delete tag category");
    let tag_cats = client
        .request()
        .list_tag_categories()
        .await
        .expect("Could not list tag categories again");
    assert_eq!(tag_cats.results.len(), 1);
}

#[instrument(skip(client))]
async fn test_tags(client: &SzurubooruClient) {
    info!("Testing tag functions");

    info!("Listing tags");
    let tag_list = client
        .request()
        .list_tags(None)
        .await
        .expect("Could not list tags");
    assert_eq!(tag_list.total, 0);

    info!("Creating tag");
    let cutag = CreateUpdateTagBuilder::default()
        .names(vec!["foo".to_string(), "foo2".to_string()])
        .category("default".to_string())
        .description("The foo tag".to_string())
        .build()
        .expect("Could not build the CreateUpdateTag");
    let tag_res = client
        .request()
        .create_tag(&cutag)
        .await
        .expect("Could not create tag");

    assert_eq!(
        tag_res.names,
        Some(vec!["foo".to_string(), "foo2".to_string()])
    );
    assert!(tag_res.description.is_some());

    let tag_list = client
        .request()
        .list_tags(None)
        .await
        .expect("Could not list tags");
    assert_eq!(tag_list.total, 1);

    info!("Testing field selection");
    let tag_list = client
        .with_fields(vec![
            "version".to_string(),
            "names".to_string(),
            "category".to_string(),
        ])
        .list_tags(None)
        .await
        .expect("Could not list tags");
    assert_eq!(tag_list.total, 1);
    assert!(tag_list.results.first().unwrap().description.is_none());

    info!("Updating tag");
    let utag = CreateUpdateTagBuilder::default()
        .version(tag_res.version)
        .description("The foo2 tag".to_string())
        .build()
        .expect("Could not build the CreateUpdateTag");
    let tag_res2 = client
        .request()
        .update_tag("foo", &utag)
        .await
        .expect("Could not update tag");
    assert_ne!(tag_res.description, tag_res2.description);

    info!("Getting tag");
    let tag_res3 = client
        .request()
        .get_tag("foo")
        .await
        .expect("Could not fetch tag");
    assert_eq!(tag_res2.description, tag_res3.description);

    info!("Creating a second tag");
    let cutag = CreateUpdateTagBuilder::default()
        .names(vec!["bar".to_string()])
        .category("default".to_string())
        .description("The foo tag".to_string())
        .build()
        .expect("Could not build the CreateUpdateTag");
    let bar_tag = client
        .request()
        .create_tag(&cutag)
        .await
        .expect("Unable to create second tag");
    let tag_list = client
        .request()
        .list_tags(None)
        .await
        .expect("Could not list tags");
    assert_eq!(tag_list.total, 2);

    info!("Merging tags");
    let merge_tag = MergeTagsBuilder::default()
        .remove_tag_version(bar_tag.version)
        .remove_tag(bar_tag.names.as_ref().unwrap().first().unwrap().clone())
        .merge_to_version(tag_res3.version)
        .merge_to_tag(tag_res3.names.as_ref().unwrap().first().unwrap().clone())
        .build()
        .expect("Could not create merge tags object");
    let merged_tag = client
        .request()
        .merge_tags(&merge_tag)
        .await
        .expect("Could not merge tags");
    assert_eq!(tag_res3.names, merged_tag.names);
    let tag_list = client
        .request()
        .list_tags(None)
        .await
        .expect("Could not list tags");
    assert_eq!(tag_list.total, 1);

    info!("Deleting tag");
    client
        .request()
        .delete_tag("foo", merged_tag.version)
        .await
        .expect("Could not delete tag");
}

#[instrument(skip(client))]
async fn test_creating_posts(client: &SzurubooruClient) {
    info!("Testing creating posts");

    info!("Listing posts (should be empty)");
    let post_list = client
        .request()
        .list_posts(None)
        .await
        .expect("Could not list posts");
    assert_eq!(post_list.total, 0);

    info!("Testing upload by URL");
    let wiki_post_obj = CreateUpdatePostBuilder::default()
        .content_url("https://upload.wikimedia.org/wikipedia/commons/thumb/5/5a/Maine_Coon_cat_by_Tomitheos.JPG/225px-Maine_Coon_cat_by_Tomitheos.JPG".to_string())
        .tags(vec!["maine_coon".to_string(), "cat".to_string()])
        .safety(PostSafety::Safe)
        .build()
        .expect("Could not build wiki post object");
    let wiki_post = client
        .request()
        .create_post_from_url(&wiki_post_obj)
        .await
        .expect("Unable to create wiki post object");

    info!("Updating existing post");
    let wiki_post_update = CreateUpdatePostBuilder::default()
        .version(wiki_post.version.unwrap())
        .safety(wiki_post.safety.unwrap())
        .source("Wikipedia".to_string())
        .build()
        .expect("Could not build wiki post update object");
    let wiki_post = client
        .request()
        .update_post(wiki_post.id.unwrap(), &wiki_post_update)
        .await
        .expect("Unable to up wiki post object");
    let post_list = client
        .request()
        .list_posts(None)
        .await
        .expect("Could not list posts");
    assert_eq!(post_list.total, 1);

    info!("Deleting wikipedia image");
    client
        .request()
        .delete_post(wiki_post.id.unwrap(), wiki_post.version.unwrap())
        .await
        .expect("Could not delete wiki post");

    info!("Test upload by File type");
    let folly1_obj = CreateUpdatePostBuilder::default()
        .tags(vec![
            "maine_coon".to_string(),
            "cat".to_string(),
            "folly1".to_string(),
        ])
        .safety(PostSafety::Safe)
        .build()
        .expect("Could not build first upload object");
    let folly1_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../folly1.jpg");
    let mut folly1_file =
        File::open(&folly1_path).expect(&format!("Could not open file {folly1_path:?}"));
    let _folly1_post = client
        .request()
        .create_post_from_file(&mut folly1_file, None, "folly1.jpg", &folly1_obj)
        .await
        .expect("Could not create post from folly1 file");

    info!("Test upload by file path");
    let folly2_obj = CreateUpdatePostBuilder::default()
        .tags(vec![
            "maine_coon".to_string(),
            "cat".to_string(),
            "folly2".to_string(),
        ])
        .safety(PostSafety::Safe)
        .build()
        .expect("Could not build second upload object");
    let folly2_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../folly2.jpg");
    let _folly2_post = client
        .request()
        .create_post_from_file_path(folly2_path, None::<String>, &folly2_obj)
        .await
        .expect("Could not create post from folly2 path");

    info!("Test upload by file path with thumbnail");
    let folly3_obj = CreateUpdatePostBuilder::default()
        .tags(vec![
            "maine_coon".to_string(),
            "cat".to_string(),
            "folly3".to_string(),
        ])
        .safety(PostSafety::Safe)
        .build()
        .expect("Could not build third upload object");
    let folly3_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../folly3.jpg");
    let folly3_thumbnail = Path::new(env!("CARGO_MANIFEST_DIR")).join("../folly3_thumb.jpg");
    let folly3_post = client
        .request()
        .create_post_from_file_path(&folly3_path, Some(folly3_thumbnail), &folly3_obj)
        .await
        .expect("Could not create post with thumbnail");

    info!("Searching for post by image");
    let matching_posts = client
        .request()
        .post_for_file_path(&folly3_path)
        .await
        .expect("Could not search for post by file path");
    assert_eq!(&matching_posts.unwrap(), &folly3_post);

    info!("Reverse searching");
    let matching_posts = client
        .request()
        .reverse_search_file_path(&folly3_path)
        .await
        .expect("Could not reverse image search");
    assert!(matching_posts.exact_post.is_some());

    info!("Testing temporary upload");
    let folly4_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../folly4.jpg");
    let folly4_temp_upload = client
        .request()
        .upload_temporary_file_from_path(folly4_path)
        .await
        .expect("Could not create temporary upload for folly4");
    let folly4_obj = CreateUpdatePostBuilder::default()
        .tags(vec![
            "maine_coon".to_string(),
            "cat".to_string(),
            "folly4".to_string(),
        ])
        .content_token(folly4_temp_upload.token)
        .safety(PostSafety::Safe)
        .build()
        .expect("Could not build fourth upload object");
    let folly4_post = client
        .request()
        .create_post_from_token(&folly4_obj)
        .await
        .expect("Could not create upload from temporary token");

    info!("Querying by tag");
    let f4_results = client
        .request()
        .list_posts(Some(&vec![QueryToken::anonymous("cat")]))
        .await
        .expect("Could not list posts by tag cat");
    assert_eq!(f4_results.total, 4);

    info!("Testing pagination");
    let post_list = client
        .request()
        .with_limit(1)
        .list_posts(None)
        .await
        .expect("Could not list posts page 1");
    assert_eq!(post_list.results.len(), 1);
    let post_list2 = client
        .request()
        .with_limit(1)
        .with_offset(1)
        .list_posts(None)
        .await
        .expect("Could not list posts page 2");
    assert_ne!(post_list.results, post_list2.results);

    info!("Testing tag siblings");
    let tag_occurrences = client
        .request()
        .get_tag_siblings("maine_coon")
        .await
        .expect("Could not fetch tag siblings");
    let occurrences_filtered = tag_occurrences
        .results
        .iter()
        .filter(|oc| {
            oc.tag
                .names
                .as_ref()
                .map(|names| names.contains(&"cat".to_string()))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(occurrences_filtered, 1);

    info!("Rating post");
    let folly3_post = client
        .request()
        .rate_post(folly3_post.id.unwrap(), 1)
        .await
        .expect("Could not rate post");
    assert_eq!(folly3_post.own_score, Some(1));

    info!("Favoriting post");
    let folly3_post = client
        .request()
        .favorite_post(folly3_post.id.unwrap())
        .await
        .expect("Could not favorite post");
    assert_eq!(folly3_post.own_favorite, Some(true));

    info!("Unfavorite post");
    let folly3_post = client
        .request()
        .unfavorite_post(folly3_post.id.unwrap())
        .await
        .expect("Could not unfavorite post");
    assert_eq!(folly3_post.own_favorite, Some(false));

    info!("Featuring post");
    let featured_post = client
        .request()
        .get_featured_post()
        .await
        .expect("Could not get featured post");
    assert!(featured_post.is_none());

    client
        .request()
        .set_featured_post(folly4_post.id.unwrap())
        .await
        .expect("Could not set featured post");

    let featured_post = client
        .request()
        .get_featured_post()
        .await
        .expect("Could not get featured post");

    assert!(featured_post.is_some());
}

#[instrument(skip(client))]
async fn test_pool_categories(client: &SzurubooruClient) {
    info!("Testing pool categories");

    info!("Listing pool categories");
    let pool_cats = client
        .request()
        .list_pool_categories()
        .await
        .expect("Could not list pool categories");
    assert!(!pool_cats.results.is_empty());

    info!("Creating pool category");
    let create_cat = CreateUpdatePoolCategoryBuilder::default()
        .name("cat_pool_category".to_string())
        .color("purple".to_string())
        .build()
        .expect("Could not build pool category object");
    let pool_cat = client
        .request()
        .create_pool_category(&create_cat)
        .await
        .expect("Could not create pool category");

    let create_dog_cat = CreateUpdatePoolCategoryBuilder::default()
        .name("dog_category".to_string())
        .color("orange".to_string())
        .build()
        .expect("Could not build pool category object");
    let dog_pool_cat = client
        .request()
        .create_pool_category(&create_dog_cat)
        .await
        .expect("Could not create pool category");

    info!("Updating pool category");
    let update_cat = CreateUpdatePoolCategoryBuilder::default()
        .version(pool_cat.version.unwrap())
        .color("white".to_string())
        .build()
        .expect("Could not build pool category update");
    let pool_cat = client
        .request()
        .update_pool_category(pool_cat.name.unwrap(), &update_cat)
        .await
        .expect("Could not update pool category");

    info!("Getting pool category");
    let pool_cat = client
        .request()
        .get_pool_category(pool_cat.name.unwrap())
        .await
        .expect("Could not get pool category");

    info!("Deleting pool category");
    client
        .request()
        .delete_pool_category(dog_pool_cat.name.unwrap(), dog_pool_cat.version.unwrap())
        .await
        .expect("Could not delete pool category");

    info!("Setting default pool category");
    let _pool_cat = client
        .request()
        .set_default_pool_category(pool_cat.name.unwrap())
        .await
        .expect("Could not set default pool category");
}

#[instrument(skip(client))]
async fn test_pools(client: &SzurubooruClient) {
    info!("Testing post pools");
    let pools = client
        .request()
        .list_pools(None)
        .await
        .expect("Could not list pools");
    assert_eq!(pools.total, 0);

    info!("Creating pools");
    let create_pool = CreateUpdatePoolBuilder::default()
        .names(vec!["cats_pool".to_string()])
        .category("cat_pool_category".to_string())
        .build()
        .expect("Could not build pool creation object");
    let cat_pool = client
        .request()
        .create_pool(&create_pool)
        .await
        .expect("Could not create pool");
    let create_pool2 = CreateUpdatePoolBuilder::default()
        .names(vec!["catz_pool".to_string()])
        .category("cat_pool_category".to_string())
        .build()
        .expect("Could not build pool creation object");
    let catz_pool = client
        .request()
        .create_pool(&create_pool2)
        .await
        .expect("Could not create pool");
    let create_pool3 = CreateUpdatePoolBuilder::default()
        .names(vec!["dogs_pool".to_string()])
        .category("cat_pool_category".to_string())
        .build()
        .expect("Could not build pool creation object");
    let dogs_pool = client
        .request()
        .create_pool(&create_pool3)
        .await
        .expect("Could not create pool");

    info!("Getting pool");
    let cat_pool = client
        .request()
        .get_pool(cat_pool.id.unwrap())
        .await
        .expect("Could not fetch pool");

    info!("Deleting pool");
    client
        .request()
        .delete_pool(dogs_pool.id.unwrap(), dogs_pool.version.unwrap())
        .await
        .expect("Could not delete pool");

    info!("Updating pool");
    let f4_results = client
        .request()
        .list_posts(Some(&vec![QueryToken::anonymous("cat")]))
        .await
        .expect("Could not list posts by tag cat");
    let post_ids = f4_results
        .results
        .into_iter()
        .map(|p| p.id.unwrap())
        .collect::<Vec<u32>>();
    let update_pool = CreateUpdatePoolBuilder::default()
        .version(cat_pool.version.unwrap())
        .posts(post_ids)
        .description("All cat pictures all the time".to_string())
        .build()
        .expect("Could not build update object");
    let cat_pool = client
        .request()
        .update_pool(cat_pool.id.unwrap(), &update_pool)
        .await
        .expect("Unable to update pool");

    info!("Merging pools");
    let merge_pool_obj = MergePoolBuilder::default()
        .remove_pool_version(catz_pool.version.unwrap())
        .remove_pool(catz_pool.id.unwrap())
        .merge_to_version(cat_pool.version.unwrap())
        .merge_to_pool(cat_pool.id.unwrap())
        .build()
        .expect("Unable to build merge object");
    let _cat_pool = client
        .request()
        .merge_pools(&merge_pool_obj)
        .await
        .expect("Unable to merge pools");
}

#[instrument(skip(client))]
async fn test_comments(client: &SzurubooruClient) {
    info!("Testing comments");

    info!("Listing comments");
    let _comment_list = client
        .request()
        .list_comments(None)
        .await
        .expect("Unable to list comments");

    let f4_results = client
        .request()
        .list_posts(Some(&vec![QueryToken::anonymous("cat")]))
        .await
        .expect("Could not list posts by tag cat");
    let post_ids = f4_results
        .results
        .into_iter()
        .map(|p| p.id.unwrap())
        .collect::<Vec<u32>>();
    let post_id = *post_ids.first().unwrap();

    info!("Creating comment");
    let create_comment = CreateUpdateCommentBuilder::default()
        .text("Excellent cat!".to_string())
        .post_id(post_id)
        .build()
        .expect("Unable to create comment object");
    let comment = client
        .request()
        .create_comment(&create_comment)
        .await
        .expect("Unable to create comment");

    info!("Updating comment");
    let update_comment = CreateUpdateCommentBuilder::default()
        .text("Beautiful cat!".to_string())
        .version(comment.version.unwrap())
        .build()
        .expect("Unable to create comment update object");
    let comment = client
        .request()
        .update_comment(comment.id.unwrap(), &update_comment)
        .await
        .expect("Unable to update comment");

    info!("Getting comment");
    let comment = client
        .request()
        .get_comment(comment.id.unwrap())
        .await
        .expect("Unable to fetch comment");

    info!("Getting all comments for post");
    let comment_list = client
        .request()
        //.list_comments(None)
        .list_comments(Some(&vec![QueryToken::token(
            CommentNamedToken::Post,
            post_id.to_string(),
        )]))
        .await
        .expect("Unable to fetch comments for post");
    assert_ne!(comment_list.total, 0);

    info!("Rating comment");
    let comment = client
        .request()
        .rate_comment(comment.id.unwrap(), -1)
        .await
        .expect("Could not rate comment");
    assert_eq!(comment.own_score, Some(-1));

    info!("Deleting comment");
    client
        .request()
        .delete_comment(comment.id.unwrap(), comment.version.unwrap())
        .await
        .expect("Could not delete comment");
}

#[instrument(skip(client))]
async fn test_users(client: &SzurubooruClient) {
    info!("Testing users");

    info!("Listing users");
    let user_list = client
        .request()
        .list_users(None)
        .await
        .expect("Could not list users");
    assert_eq!(user_list.total, 1);

    // Create user is already tested above
    info!("Creating user with avatar");
    let avatar_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../avatar.jpg");
    let create_user = CreateUpdateUserBuilder::default()
        .name("iu2".to_string())
        .password("ipass2".to_string())
        .rank(UserRank::Regular)
        .avatar_style(UserAvatarStyle::Manual)
        .build()
        .expect("Could not create user creation object");
    let user_obj = client
        .request()
        .create_user_with_avatar_path(avatar_path, &create_user)
        .await
        .expect("Could not create user");

    info!("Updating user");
    let update_user = CreateUpdateUserBuilder::default()
        .version(user_obj.version.unwrap())
        .rank(UserRank::Restricted)
        .build()
        .expect("Could not create user update object");
    let user_obj = client
        .request()
        .update_user(user_obj.name.unwrap(), &update_user)
        .await
        .expect("Could not update user");

    info!("Getting user");
    let user_obj = client
        .request()
        .get_user(user_obj.name.unwrap())
        .await
        .expect("Could not get user");

    info!("Deleting user");
    client
        .request()
        .delete_user(user_obj.name.unwrap(), user_obj.version.unwrap())
        .await
        .expect("Could not delete user");

    let username = "integration_user".to_string();
    info!("Listing user tokens");
    let tokens = client
        .request()
        .list_user_tokens(&username)
        .await
        .expect("Could not list user tokens");
    assert!(tokens.results.is_empty());

    info!("Creating user token");
    let create_token = CreateUpdateUserAuthTokenBuilder::default()
        .note("My token")
        .enabled(true)
        .build()
        .expect("Could not create token creation object");
    let token = client
        .request()
        .create_user_token(&username, &create_token)
        .await
        .expect("Could not create auth token");

    info!("Updating user token");
    let new_expiration = chrono::offset::Utc::now()
        .checked_add_months(Months::new(1))
        .unwrap();
    let update_token = CreateUpdateUserAuthTokenBuilder::default()
        .version(token.version.unwrap())
        .expiration_time(new_expiration)
        .build()
        .expect("Could not create token update object");

    let token = client
        .request()
        .update_user_token(&username, &token.token.unwrap(), &update_token)
        .await
        .expect("Could not update token");

    info!("Deleting user token");
    client
        .request()
        .delete_user_token(username, token.token.unwrap(), token.version.unwrap())
        .await
        .expect("Could not delete token");
}

#[instrument(skip(client))]
async fn test_snapshots(client: &SzurubooruClient) {
    info!("Testing snapshots");

    info!("Listing snapshots");
    let snap_list = client
        .request()
        .list_snapshots(None)
        .await
        .expect("Could not list snapshots");
    assert!(snap_list.total > 0);
}

#[instrument(skip(client))]
async fn test_downloads(client: &SzurubooruClient) {
    info!("Testing image download");

    let folly3_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../folly3.jpg");

    let mut f3_hasher = Sha1::new();

    let mut f3_file = File::open(&folly3_path).expect("Unable to open folly3.jpg");

    std::io::copy(&mut f3_file, &mut f3_hasher).expect("Unable to hash folly3.jpg");

    let f3_hash = f3_hasher.finalize();

    let f3post = client
        .request()
        .reverse_search_file_path(folly3_path)
        .await
        .expect("Unable to reverse search for file")
        .exact_post
        .expect("Unable to find exact post for folly3.jpg");

    let mut dl_file = tempfile().expect("Could not create temporary file");
    //let mut dl_thumb_file = tempfile().expect("Could not create temporary thumbnail file");

    //let dlfolly3_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("dlfolly3.jpg");
    /*let mut dl_file = File::options()
    .create(true)
    .truncate(true)
    .read(true)
    .write(true)
    .open(dlfolly3_path)
    .expect("unable to open download file");*/

    client
        .request()
        .download_image_to_file(f3post.id.unwrap(), &mut dl_file)
        .await
        .expect("Could not download to temporary file");
    dl_file
        .seek(SeekFrom::Start(0))
        .expect("Could not rewind file");

    let mut dl_f3_hasher = Sha1::new();

    std::io::copy(&mut dl_file, &mut dl_f3_hasher).expect("Unable to hash folly3.jpg");

    let dl_f3_hash = dl_f3_hasher.finalize();

    assert_eq!(f3_hash, dl_f3_hash);
}
