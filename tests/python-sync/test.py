from szurubooru_client import *
import time, sys
from loguru import logger
import hashlib
import tempfile, pathlib

def connect():
    logger.info("Connecting to Szurubooru instance")
    anon_client = SzurubooruSyncClient("http://localhost:9802")
    error = None
    for i in range(5):
        try:
            anon_client.global_info()
        except Exception as e:
            error=e
            time.sleep(5)
        else:
            logger.info("Connection successful!")
            break
    else:
        logger.error("Could not connect to Szurubooru instance, error is {}", error)
        sys.exit(1)
    return anon_client

def create_auth_client(client):
    client.create_user("integration_user", "integration_password", rank=UserRank.Administrator)
    auth_client = SzurubooruSyncClient("http://localhost:9802", username="integration_user",
                                       password="integration_password", allow_insecure=True)
    return auth_client

def test_tag_categories(client):
    logger.info("Testing tag categories")
    logger.info("Listing tag categories")
    tag_cats = client.list_tag_categories()
    assert len(tag_cats) == 1

    logger.info("Creating tag category")
    result_tag_cat = client.create_tag_category("my_tag_cat", color="purple", order=1)
    assert result_tag_cat.name == "my_tag_cat"

    tag_cats = client.list_tag_categories()
    assert len(tag_cats) != 1
    assert tag_cats[1].name == "my_tag_cat"

    logger.info("Getting tag category")
    get_tag_cat = client.get_tag_category("my_tag_cat")
    assert get_tag_cat.color == result_tag_cat.color

    logger.info("Updating tag category")
    update_tag_cat = client.update_tag_category("my_tag_cat", get_tag_cat.version, color="red")
    assert update_tag_cat.color != get_tag_cat.color

    logger.info("Deleting tag category")
    client.delete_tag_category("my_tag_cat", update_tag_cat.version)
    tag_cats = client.list_tag_categories()
    assert len(tag_cats) == 1

def test_tag(client):
    logger.info("Testing tags")
    logger.info("Listing tags")

    tags = client.list_tags()
    assert len(tags.results) == 0

    logger.info("Creating tag")
    foo_tag = client.create_tag("foo", category="default", description="The foo tag")
    assert foo_tag.names == ["foo"]
    tags = client.list_tags()
    assert len(tags.results) == 1

    logger.info("Testing field selection")
    tags = client.list_tags(fields=["version", "names", "category"])
    assert len(tags.results) == 1
    assert tags.results[0].description is None

    logger.info("Updating tag")
    foo_tag = client.update_tag(foo_tag.names[0], version=foo_tag.version, description="The foo2 tag")
    assert foo_tag.description == "The foo2 tag"

    logger.info("Getting tag")
    foo_tag2 = client.get_tag("foo")
    assert foo_tag.description == foo_tag2.description

    logger.info("Creating a second tag")
    bar_tag = client.create_tag("bar", category="default", description="The bar tag")
    assert bar_tag.names == ["bar"]

    logger.info("Merging tags")
    foo_tag2 = client.merge_tags(bar_tag.names[0], bar_tag.version,
                                 foo_tag2.names[0], foo_tag2.version)
    tags = client.list_tags()
    assert len(tags.results) == 1

    logger.info("Deleting tag")
    client.delete_tag(foo_tag2.names[0], foo_tag2.version)

def test_creating_posts(client):
    logger.info("Testing posts")

    logger.info("Listing posts")
    posts = client.list_posts()
    assert len(posts.results) == 0

    logger.info("Creating post from URL")
    wiki_post = client.create_post(url="https://upload.wikimedia.org/wikipedia/commons/thumb/5/5a/Maine_Coon_cat_by_Tomitheos.JPG/225px-Maine_Coon_cat_by_Tomitheos.JPG",
                                   safety=PostSafety.Safe)
    posts = client.list_posts()
    assert len(posts.results) == 1

    logger.info("Updating post")
    wiki_post = client.update_post(wiki_post.id, wiki_post.version, source="Wikipedia")
    assert wiki_post.source == "Wikipedia"

    logger.info("Deleting post")
    client.delete_post(wiki_post.id, wiki_post.version)
    posts = client.list_posts()
    assert len(posts.results) == 0

    logger.info("Testing upload by file path")
    folly1 = client.create_post(file_path="../folly1.jpg",
                                tags=["maine_coon", "cat", "folly1"],
                                safety=PostSafety.Safe)
    posts = client.list_posts()
    assert len(posts.results) == 1

    folly2 = client.create_post(file_path="../folly2.jpg",
                                tags=["maine_coon", "cat", "folly2"],
                                safety=PostSafety.Safe)

    logger.info("Testing upload with thumbnail")
    folly3 = client.create_post(file_path="../folly3.jpg",
                                thumbnail_path="../folly3_thumb.jpg",
                                tags=["maine_coon", "cat", "folly3"],
                                safety=PostSafety.Safe)

    logger.info("Searching for a post with image")
    folly3_search = client.post_for_image("../folly3.jpg")
    assert folly3_search is not None
    assert folly3_search.id == folly3.id

    logger.info("Reverse image searching")
    reverse_search = client.reverse_image_search("../folly3.jpg")
    assert reverse_search.exact_post.id == folly3.id

    logger.info("Testing temporary upload")
    token = client.upload_temporary_file("../folly4.jpg")
    folly4 = client.create_post(token=token, tags=["maine_coon", "cat", "folly4"],
                                safety=PostSafety.Safe)

    logger.info("Querying by anonymous tag")
    cat_posts = client.list_posts(query=[anonymous_token("cat")])
    assert len(cat_posts.results) == 4

    logger.info("Querying by named tag")
    cat_posts = client.list_posts(query=[named_token(PostNamedToken.Tag, "maine_coon")])
    assert len(cat_posts.results) == 4

    logger.info("Testing pagination")
    cat_posts = client.list_posts(limit=1)
    assert cat_posts.total == 4
    assert len(cat_posts.results) == 1

    cat_posts2 = client.list_posts(limit=1, offset=1)
    assert cat_posts.results != cat_posts2.results

    logger.info("Testing tag siblings")
    tag_occurrences = client.get_tag_siblings("maine_coon")
    cat_occurrences = list(filter(lambda x: "cat" in x.tag.names, tag_occurrences))
    assert len(cat_occurrences) == 1

    logger.info("Rating post")
    client.rate_post(folly3.id, 1)

    logger.info("Testing rating error validation")
    try:
        client.rate_post(folly3.id, -2)
    except ValueError:
        assert True
    else:
        assert False

    logger.info("Favoriting post")
    client.favorite_post(folly3.id)

    logger.info("Unfavoriting post")
    client.unfavorite_post(folly3.id)

    logger.info("Featuring post")
    client.set_featured_post(folly3.id)

    logger.info("Getting featured post")
    featured_post = client.get_featured_post()
    assert folly3.id == featured_post.id

    logger.info("Merging posts")
    merged_post = client.merge_post(folly4.id, folly4.version, folly3.id, folly3.version)
    assert merged_post.id == folly3.id

def test_pool_categories(client):
    logger.info("Testing pool categories")

    logger.info("Listing pool categories")
    pool_cats = client.list_pool_categories()
    assert len(pool_cats) != 0

    logger.info("Creating pool category")
    pool_cat = client.create_pool_category("cat_pool_category", color="purple")
    assert pool_cat.color == "purple"
    pool_dog = client.create_pool_category("dog_category", color="orange")

    logger.info("Updating pool category")
    pool_cat = client.update_pool_category(pool_cat.name, pool_cat.version, color="white")
    assert pool_cat.color == "white"

    logger.info("Getting pool category")
    pool_dog = client.get_pool_category(pool_dog.name)
    assert pool_dog.color == "orange"

    logger.info("Deleting pool category")
    pool_dog = client.delete_pool_category(pool_dog.name, pool_dog.version)

    logger.info("Setting default pool category")
    client.set_default_pool_category(pool_cat.name)

def test_pools(client):
    logger.info("Testing post pools")
    logger.info("Listing post pools")
    pools = client.list_pools()
    assert len(pools.results) == 0

    logger.info("Creating pools")
    cat_pool = client.create_pool("cats_pool", category="cat_pool_category")
    catz_pool = client.create_pool("catz_pool", category="cat_pool_category")
    dogs_pool = client.create_pool("dogs_pool", category="cat_pool_category")

    logger.info("Getting pool")
    cat_pool = client.get_pool(cat_pool.id)

    logger.info("Deleting pool")
    client.delete_pool(dogs_pool.id, dogs_pool.version)

    logger.info("Updating pool")
    f4_results = client.list_posts([anonymous_token("cat")])
    post_ids= list(map(lambda p: p.id, f4_results.results))
    cat_pool = client.update_pool(cat_pool.id, cat_pool.version,
                                  posts=post_ids, description="All cat pictures")
    assert len(cat_pool.posts) != 0

    logger.info("Merging pools")
    merged_pool = client.merge_pools(catz_pool.id, catz_pool.version, cat_pool.id, cat_pool.version)
    assert merged_pool.id == cat_pool.id

def test_comments(client):
    logger.info("Testing post comments")

    logger.info("Listing post comments")
    comment_list = client.list_comments()
    assert len(comment_list.results) == 0

    cat_results = client.list_posts([anonymous_token("cat")])
    post_id = cat_results.results[0].id

    logger.info("Creating comment")
    comment = client.create_comment("Excellent cat!", post_id)

    logger.info("Updating comment")
    comment = client.update_comment(comment.id, comment.version, text="Beautiful cat!")
    assert comment.text == "Beautiful cat!"

    logger.info("Getting comment")
    comment = client.get_comment(comment.id)
    assert comment.text == "Beautiful cat!"

    logger.info("Getting all comments for post")
    comment_list = client.list_comments([named_token(CommentNamedToken.Post, post_id)])
    assert len(comment_list.results) != 0

    logger.info("Testing rating comments")
    comment = client.rate_comment(comment.id, -1)
    assert comment.own_score == -1

    try:
        client.rate_comment(comment.id, -2)
    except SzuruPyClientError:
        assert True
    else:
        assert False

    logger.info("Deleting comment")
    client.delete_comment(comment.id, comment.version)

def test_users(client):
    logger.info("Testing users")

    logger.info("Listing users")
    user_list = client.list_users()

    logger.info("Creating user with avatar")
    user = client.create_user("iu2", "ipass2", rank=UserRank.Regular, avatar_path="../avatar.jpg")
    assert user.avatar_style == UserAvatarStyle.Manual

    logger.info("Updating user")
    user = client.update_user(user.name, user.version, rank=UserRank.Restricted)

    logger.info("Getting user")
    user = client.get_user(user.name)

    logger.info("Deleting user")
    client.delete_user(user.name, user.version)

    logger.info("Listing user tokens")
    tokens = client.list_user_tokens("integration_user")
    assert len(tokens) == 0

    logger.info("Creating user token")
    token = client.create_user_token("integration_user", "My token")

    logger.info("Updating user token")
    token2 = client.update_user_token("integration_user", token.token, token.version, enabled=False)
    assert token2.enabled == False

    logger.info("Deleting user token")
    client.delete_user_token("integration_user", token2.token, token2.version)

def test_snapshots(client):
    logger.info("Testing snapshots")

    logger.info("Listing snapshots")
    snapshot_list = client.list_snapshots()
    assert len(snapshot_list.results) != 0

def test_downloads(client):
    logger.info("Testing downloads")
    f3_hasher = hashlib.new("sha1")

    with open("../folly3.jpg", "rb") as f:
        while (byte := f.read(1)):
            f3_hasher.update(byte)

    f3_post = client.list_posts([anonymous_token("folly3")]).results[0]
    with tempfile.TemporaryDirectory() as tmpdirname:
        tmpdir = pathlib.Path(tmpdirname)
        fname = tmpdir / "folly3_dl.jpg"
        #fname = "../folly4_dl.jpg"
        client.download_image_to_path(f3_post.id, fname)
        dl_hasher = hashlib.new("sha1")
        with open(fname, "rb") as f:
            while (byte := f.read(1)):
             dl_hasher.update(byte)

    assert f3_hasher.hexdigest() == dl_hasher.hexdigest()

if __name__ == "__main__":
    client = connect()
    client = create_auth_client(client)
    test_tag_categories(client)
    test_tag(client)
    test_creating_posts(client)
    test_pool_categories(client)
    test_pools(client)
    test_comments(client)
    test_users(client)
    test_snapshots(client)
    test_downloads(client)
