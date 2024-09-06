# szurubooru-client

SzurubooruClient is a wrapper around the excellently-documented Szurubooru API,
including type-safe (if not API-safe) Query and Sort tokens. Includes a Python wrapper.

## From Rust

### Creating a new client

#### Basic authentication
Please keep in mind that this is not the preferred method of authentication. Tokens
are far superior.

```rust
use szurubooru_client::SzurubooruClient;
let client = SzurubooruClient::new_with_basic_auth("http://localhost:5001", "myuser",
    "mypassword", true).unwrap();
```

#### Token authentication
The far superior and more secure means of authentication

```rust
use szurubooru_client::SzurubooruClient;
let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
```

For all other methods for making the requests, see the [documentation](https://docs.rs/szurubooru-client/latest).

## From Python

Install the client using `pip install szurubooru-client`

```python
from szurubooru_client import *
from szurubooru_client.tokens import *
from szurubooru_client.models import *
```

### Creating a client


```python
client = SzurubooruSyncClient("http://localhost:9802", username="integration_user",
                                       password="integration_password", allow_insecure=True)
```

### Creating a new tag


```python
foo_tag = client.create_tag("foo", category="default", description="The foo tag")
assert foo_tag.names == ["foo"]
```

### Returning only a subset of fields


```python
# Omit the "description" field
tags = client.list_tags(fields=["version", "names", "category"])
assert tags.results[0].description is None
```

### Uploading from a file path


```python
folly1 = client.create_post(file_path="../folly1.jpg",
                            tags=["maine_coon", "cat", "folly1"],
                            safety=PostSafety.Safe)
```

### Searching for an existing post using an image


```python
folly1_search = client.post_for_image("../folly1.jpg")
assert folly1_search is not None
```

### Querying by an anonymous tag


```python
cat_posts = client.list_posts(query=[anonymous_token("cat")])
```

### Querying by a named tag


```python
mc_posts = client.list_posts(query=[named_token(PostNamedToken.Tag, "maine_coon")])
```

### Pagination


```python
posts = client.list_posts(limit=1)
assert posts.total == 4
assert len(posts.results) == 1

posts2 = client.list_posts(limit=1, offset=1)
assert posts.results != posts2.results
```

### Commenting on a post


```python
cat_results = client.list_posts([anonymous_token("cat")])
post_id = cat_results.results[0].id

comment = client.create_comment("Excellent cat!", post_id)
```

### Getting all comments for a post


```python
comment_list = client.list_comments([named_token(CommentNamedToken.Post, post_id)])
assert len(comment_list.results) != 0
```

### Downloading an image to a local path


```python
cat_results = client.list_posts([anonymous_token("cat")])
post_id = cat_results.results[0].id

client.download_image_to_path(post_id, "/tmp/cat.jpg")
```
