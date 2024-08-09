#[cfg(test)]
mod tests {
    use crate::client::SzurubooruClient;
    use crate::errors::*;
    use crate::models::*;
    use mockito::Server;
    use serde_json::{json, to_string};

    static TEST_AUTH: &str = "Token dGVzdDp0ZXN0";
    static TEST2_AUTH: &str = "Token dGVzdDI6dGVzdAo=";
    static ADMIN_AUTH: &str = "Token YWRtaW46dGVzdA==";

    #[tokio::test]
    async fn test_tag_categories() -> SzurubooruResult<()> {
        let mut server = Server::new_async().await;

        let test_client = SzurubooruClient::new_with_token(&server.url(), "admin", "test", true)?;
        let admin_get_tcs = server
            .mock("GET", "/tag-categories")
            .match_header("Authorization", ADMIN_AUTH)
            .with_body(
                to_string(&json!({
                    "results": [
                        {
                            "version": 0,
                            "name": "my-tag-category",
                            "color": "blue",
                            "usages": 1,
                            "order": "asc",
                            "default": false
                        }
                    ]
                }))
                .unwrap(),
            )
            .create_async()
            .await;
        let tc_res = test_client.request().list_tag_categories().await?;
        admin_get_tcs.assert_async().await;
        assert_eq!(tc_res.results.len(), 1);
        assert_eq!(
            tc_res.results.first().unwrap().color,
            Some("blue".to_string())
        );

        let fail_tcs = server
            .mock("GET", "/tag-categories")
            .match_header("Authorization", TEST_AUTH)
            .with_body(
                to_string(&json!({
                    "name": "AuthError",
                    "title": "Auth Error",
                    "description": "Authentication error"
                }))
                .unwrap(),
            )
            .create_async()
            .await;

        let test_client = SzurubooruClient::new_with_token(&server.url(), "test", "test", true)?;
        let failed_tc_res = test_client.request().list_tag_categories().await;
        fail_tcs.assert_async().await;
        match failed_tc_res {
            Err(SzurubooruClientError::SzurubooruServerError(e)) => {
                assert_eq!(e.name, SzurubooruServerErrorType::AuthError)
            }
            _ => assert!(false),
        }

        Ok(())
    }
}
