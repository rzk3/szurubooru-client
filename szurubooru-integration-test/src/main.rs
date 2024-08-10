use std::error::Error;
use std::process::Stdio;
use std::time::Duration;
use szurubooru_client::models::*;
use szurubooru_client::*;
use tokio::process::Command;
use tracing::level_filters::LevelFilter;
use tracing::{error, info, instrument};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

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
        .name("integration_user")
        .password("integration_password")
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
    info!("Starting Szurubooru instance...");
    let anon_client = SzurubooruClient::new_anonymous("http://localhost:9801", true)
        .expect("Can't create anonymous client");

    Command::new("sh")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("-c")
        .arg("./start_szurubooru.sh")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .status()
        .await
        .expect("Failed to start szurubooru");

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
        .with_fields(vec!["version", "names", "category"])
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
        .remove_version(bar_tag.version)
        .remove(bar_tag.names.as_ref().unwrap().first().unwrap())
        .merge_to_version(tag_res3.version)
        .merge_to(tag_res3.names.as_ref().unwrap().first().unwrap())
        .build()
        .expect("Could not create merge tags object");
    let merged_tag = client
        .request()
        .merge_tag(&merge_tag)
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

//TODO: test Tag siblings once we've tested a post
