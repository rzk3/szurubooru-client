Szurubooru Query Tokens
=======================

.. contents:: Table of Contents
    :depth: 3
    :local:

Most of the ``list`` methods on the client support a ``query`` parameter. This parameter (a ``list`` of tokens) allows you to filter the results returned by the API.

There's four kinds of tokens: ``Named``, ``Sort``, ``Anonymous`` and ``Special``.
Because this library is a wrapper around a Rust library, there's a group of type-safe ``Enum`` objects that allow you to
make sure you get the right field names when constructing the tokens.

All tokens can be mixed and matched to filter the results how you'd prefer.


.. _named-tokens:


Named tokens
^^^^^^^^^^^^

.. autofunction:: szurubooru_client.tokens.named_token

.. _sort-tokens:


Sort tokens
^^^^^^^^^^^

.. autofunction:: szurubooru_client.tokens.sort_token


Anonymous tokens
^^^^^^^^^^^^^^^^

.. autofunction:: szurubooru_client.tokens.anonymous_token


Special tokens
^^^^^^^^^^^^^^

.. autofunction:: szurubooru_client.tokens.special_token


Negating tokens
^^^^^^^^^^^^^^^

All query tokens can be negated using either ``.negate`` or by prefixing with the ``-`` operator. Negated tokens mean to *not* match that particular token.

For example, to match the anonymous tag ``konosuba``, we would do the following:

```python
client.list_posts(query=[anonymous_token("konosuba")])
```

Now to match all posts that *don't* have the ``konosuba`` tag:

```python
client.list_posts(query=[-anonymous_token("konosuba")])
# or
client.list_posts(query=[anonymous_token("konosuba").negate()])
```

.. _named-tokens-enums:

Token Enums
^^^^^^^^^^^

-----------------
Named Token Enums
-----------------

.. py:class:: szurubooru_client.tokens.TagNamedToken

    .. py:attribute:: Name

        Having given name (accepts wildcards)


    .. py:attribute:: Category

        Having given category (accepts wildcards)


    .. py:attribute:: CreationDate

        Created at given date


    .. py:attribute:: LastEditDate

        Edited at given date


    .. py:attribute:: LastEditTime

        Alias of :attr:`LastEditTime <szurubooru_client.tokens.TagNamedToken.LastEditTime>`


    .. py:attribute:: EditDate

        Alias of :attr:`LastEditTime <szurubooru_client.tokens.TagNamedToken.LastEditTime>`


    .. py:attribute:: EditTime

        Alias of :attr:`LastEditTime <szurubooru_client.tokens.TagNamedToken.LastEditTime>`


    .. py:attribute:: Usages

        Used in given number of posts


    .. py:attribute:: UsageCount

        Alias of :attr:`Usages <szurubooru_client.tokens.TagNamedToken.Usages>`


    .. py:attribute:: PostCount

        Alias of :attr:`Usages <szurubooru_client.tokens.TagNamedToken.Usages>`


    .. py:attribute:: SuggestionCount

        With given number of suggestions


    .. py:attribute:: ImplicationCount

        With given number of implications

.. py:class:: szurubooru_client.tokens.PostNamedToken

    .. py:attribute:: Id

        Having given post number

    .. py:attribute:: Tag

        Having given tag (accepts wildcards)


    .. py:attribute:: Score

        Having given score


    .. py:attribute:: Uploader

        Uploaded by given user (accepts wildcards)


    .. py:attribute:: Upload

        Alias of :attr:`Uploader <szurubooru_client.tokens.PostNamedToken.Uploader>`


    .. py:attribute:: Submit

        Alias of :attr:`Uploader <szurubooru_client.tokens.PostNamedToken.Uploader>`


    .. py:attribute:: Comment

        Commented by given user (accepts wildcards)


    .. py:attribute:: Fav

        Favorited by given user (accepts wildcards)


    .. py:attribute:: Pool

        Belonging to the pool with the given id


    .. py:attribute:: TagCount

        Having given number of tags


    .. py:attribute:: CommentCount

        Having given number of comments


    .. py:attribute:: FavCount

        Favorited by given number of users


    .. py:attribute:: NoteCount

        Having given number of annotations


    .. py:attribute:: NoteText

        Having given note text (accepts wildcards)


    .. py:attribute:: RelationCount

        Having given number of relations


    .. py:attribute:: FeatureCount

        Having been featured given number of times


    .. py:attribute:: Type

        ``flash`` (or ``swf``) or ``video`` (or ``webm``). Use :attr:`models.posttype` for type-safe values


    .. py:attribute:: ContentChecksum

        Having given sha1 checksum


    .. py:attribute:: FileSize

        Having given file size (in bytes)


    .. py:attribute:: ImageWidth

        Having given image width (where applicable)


    .. py:attribute:: ImageHeight

        Having given image height (where applicable)


    .. py:attribute:: ImageArea

        Having given number of pixels (image width * image height)


    .. py:attribute:: ImageAspectRatio

        Having given aspect ratio (image width / image height)


    .. py:attribute:: ImageAr

        Alias of :attr:`ImageAspectRatio <szurubooru_client.tokens.PostNamedToken.ImageAspectRatio>`


    .. py:attribute:: Width

        Alias of :attr:`ImageWidth <szurubooru_client.tokens.PostNamedToken.ImageWidth>`


    .. py:attribute:: Height

        Alias of :attr:`ImageHeight <szurubooru_client.tokens.PostNamedToken.ImageHeight>`


    .. py:attribute:: Ar

        Alias of :attr:`ImageAspectRatio <szurubooru_client.tokens.PostNamedToken.ImageAspectRatio>`


    .. py:attribute:: AspectRatio

        Alias of :attr:`ImageAspectRatio <szurubooru_client.tokens.PostNamedToken.ImageAspectRatio>`


    .. py:attribute:: CreationDate

        Posted at given date


    .. py:attribute:: CreationTime

        Alias of :attr:`CreationDate <szurubooru_client.tokens.PostNamedToken.CreationDate>`


    .. py:attribute:: Date

        Alias of :attr:`CreationDate <szurubooru_client.tokens.PostNamedToken.CreationDate>`


    .. py:attribute:: Time

        Alias of :attr:`CreationDate <szurubooru_client.tokens.PostNamedToken.CreationDate>`


    .. py:attribute:: LastEditDate

        Edited at given date


    .. py:attribute:: LastEditTime

        Alias of :attr:`LastEditDate <szurubooru_client.tokens.PostNamedToken.LastEditDate>`


    .. py:attribute:: EditDate

        Alias of :attr:`LastEditDate <szurubooru_client.tokens.PostNamedToken.LastEditDate>`


    .. py:attribute:: EditTime

        Alias of :attr:`LastEditDate <szurubooru_client.tokens.PostNamedToken.LastEditDate>`


    .. py:attribute:: CommentDate

        Commented at given date


    .. py:attribute:: CommentTime

        Alias of :attr:`CommentDate <szurubooru_client.tokens.PostNamedToken.CommentDate>`


    .. py:attribute:: FavDate

        Last favorited at given time


    .. py:attribute:: FavTime

        Alias of :attr:`FavDate <szurubooru_client.tokens.PostNamedToken.FavDate>`


    .. py:attribute:: FeatureDate

        Featured at given date


    .. py:attribute:: FeatureTime

        Alias of :attr:`FeatureDate <szurubooru_client.tokens.PostNamedToken.FeatureDate>`


    .. py:attribute:: Safety

        Use :class:`PostSafety <szurubooru_client.models.PostSafety>` for the type-safe version


    .. py:attribute:: Rating

        Alias of :attr:`Safety <szurubooru_client.tokens.PostNamedToken.Safety>`

.. py:class:: szurubooru_client.tokens.CommentNamedToken

    .. py:attribute:: Id

		Specific comment id


	.. py:attribute:: Post

		Specific post id


	.. py:attribute:: User

		Created by given user (accepts wildcards)


	.. py:attribute:: Author

		Alias of user


	.. py:attribute:: Text

		Containing given text (accepts wildcards)


	.. py:attribute:: CreationDate

		Created at given date


	.. py:attribute:: CreationTime

		Alias of :attr:`CreationDate <szurubooru_client.tokens.CommentNamedToken.CreationDate>`


	.. py:attribute:: LastEditDate

		Whose most recent edit date matches given date


	.. py:attribute:: LastEditTime

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.CommentNamedToken.LastEditDate>`


	.. py:attribute:: EditDate

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.CommentNamedToken.LastEditDate>`


	.. py:attribute:: EditTime

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.CommentNamedToken.LastEditDate>`

.. py:class:: szurubooru_client.tokens.UserNamedToken


	.. py:attribute:: Name

		Having given name (accepts wildcards)


	.. py:attribute:: CreationDate

		Registered at given date


	.. py:attribute:: CreationTime

		Alias of :attr:`CreationDate <szurubooru_client.tokens.UserNamedToken.CreationDate>`


	.. py:attribute:: LastLoginDate

		Whose most recent login date matches given date


	.. py:attribute:: LastLoginTime

		Alias of :attr:`LastLoginDate <szurubooru_client.tokens.UserNamedToken.LastLoginDate>`


	.. py:attribute:: LoginDate

		Alias of :attr:`LastLoginDate <szurubooru_client.tokens.UserNamedToken.LastLoginDate>`


	.. py:attribute:: LoginTime

		Alias of :attr:`LastLoginDate <szurubooru_client.tokens.UserNamedToken.LastLoginDate>`

.. py:class:: szurubooru_client.tokens.SnapshotNamedToken


	.. py:attribute:: Type

		Involving given resource type


	.. py:attribute:: Id

		Involving given resource id


	.. py:attribute:: Date

		Created at given date


	.. py:attribute:: Time

		Alias of :attr:`Date <szurubooru_client.tokens.UserNamedToken.LastLoginDate>`


	.. py:attribute:: Operation

		``modified``, ``created``, ``deleted`` or ``merged``. Use :attr:`SnapshotType <szurubooru_client.models.SnapshotType>` for type-safe values


	.. py:attribute:: User

		Name of the user that created given snapshot (accepts wildcards)

.. _sort-tokens-enums:

----------------
Sort Token Enums
----------------

.. py:class:: szurubooru_client.tokens.PostSortToken

	.. py:attribute:: Random

		As random as it can get


	.. py:attribute:: Id

		Highest to lowest post number


	.. py:attribute:: Score

		Highest scored


	.. py:attribute:: TagCount

		With most tags


	.. py:attribute:: CommentCount

		Most commented first


	.. py:attribute:: FavCount

		Loved by most


	.. py:attribute:: NoteCount

		With most annotations


	.. py:attribute:: RelationCount

		With most relations


	.. py:attribute:: FeatureCount

		Most often featured


	.. py:attribute:: FileSize

		Largest files first


	.. py:attribute:: ImageWidth

		Widest images first


	.. py:attribute:: ImageHeight

		Tallest images first


	.. py:attribute:: ImageArea

		Largest images first


	.. py:attribute:: Width

		Alias of :attr:`ImageWidth <szurubooru_client.tokens.PostSortToken.imagewidth>`


	.. py:attribute:: Height

		Alias of :attr:`ImageHeight <szurubooru_client.tokens.PostSortToken.imageheight>`


	.. py:attribute:: Area

		Alias of :attr:`ImageArea <szurubooru_client.tokens.PostSortToken.imagearea>`


	.. py:attribute:: CreationDate

		Newest to oldest (pretty much same as id)


	.. py:attribute:: CreationTime

		Alias of :attr:`CreationDate <szurubooru_client.tokens.PostSortToken.creationdate>`


	.. py:attribute:: Date

		Alias of :attr:`CreationDate <szurubooru_client.tokens.PostSortToken.creationdate>`


	.. py:attribute:: Time

		Alias of :attr:`CreationDate <szurubooru_client.tokens.PostSortToken.creationdate>`


	.. py:attribute:: LastEditDate

		Like :attr:`CreationDate <szurubooru_client.tokens.PostSortToken.creationdate>`, only looks at last edit time instead


	.. py:attribute:: LastEditTime

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.PostSortToken.lasteditdate>`


	.. py:attribute:: EditDate

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.PostSortToken.lasteditdate>`


	.. py:attribute:: EditTime

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.PostSortToken.lasteditdate>`


	.. py:attribute:: CommentDate

		Recently commented by anyone


	.. py:attribute:: CommentTime

		Alias of :attr:`CommentDate <szurubooru_client.tokens.PostSortToken.commentdate>`


	.. py:attribute:: FavDate

		Recently added to favorites by anyone


	.. py:attribute:: FavTime

		Alias of :attr:`FavDate <szurubooru_client.tokens.PostSortToken.favdate>`


	.. py:attribute:: FeatureDate

		Recently featured


	.. py:attribute:: FeatureTime

		Alias of :attr:`FeatureDate <szurubooru_client.tokens.PostSortToken.featuredate>`


.. py:class:: szurubooru_client.tokens.TagSortToken

    .. py:attribute:: Random

        As random as it can get


    .. py:attribute:: Name

        A to Z


    .. py:attribute:: Category

        Category (a to z)


    .. py:attribute:: CreationDate

        Recently created first


    .. py:attribute:: CreationTime

        Alias of :attr:`CreationDate <szurubooru_client.tokens.TagSortToken.CreationDate>`


    .. py:attribute:: LastEditDate

        Recently edited first


    .. py:attribute:: LastEditTime

        Alias of :attr:`CreationDate <szurubooru_client.tokens.TagSortToken.CreationDate>`


    .. py:attribute:: EditDate

        Alias of :attr:`CreationDate <szurubooru_client.tokens.TagSortToken.CreationDate>`


    .. py:attribute:: EditTime

        Alias of :attr:`CreationDate <szurubooru_client.tokens.TagSortToken.CreationDate>`


    .. py:attribute:: Usages

        Used in most posts first


    .. py:attribute:: UsageCount

        Alias of :attr:`Usages <szurubooru_client.tokens.TagSortToken.Usages>`


    .. py:attribute:: PostCount

        Alias of :attr:`Usages <szurubooru_client.tokens.TagSortToken.Usages>`


    .. py:attribute:: SuggestionCount

        With most suggestions first


    .. py:attribute:: ImplicationCount

        With most implications first


.. py:class:: szurubooru_client.tokens.CommentSortToken


	.. py:attribute:: Random

		As random as it can get


	.. py:attribute:: User

		Author name, a to z


	.. py:attribute:: Author

		Alias of user


	.. py:attribute:: Post

		Post id, newest to oldest


	.. py:attribute:: CreationDate

		Newest to oldest


	.. py:attribute:: CreationTime

		Alias of :attr:`CreationDate <szurubooru_client.tokens.CommentSortToken.CreationDate>`


	.. py:attribute:: LastEditDate

		Recently edited first


	.. py:attribute:: LastEditTime

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.CommentSortToken.LastEditDate>`


	.. py:attribute:: EditDate

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.CommentSortToken.LastEditDate>`


	.. py:attribute:: EditTime

		Alias of :attr:`LastEditDate <szurubooru_client.tokens.CommentSortToken.LastEditDate>`


.. py:class:: szurubooru_client.tokens.UserSortToken

	.. py:attribute:: Random

		As random as it can get


	.. py:attribute:: Name

		A to z


	.. py:attribute:: CreationDate

		Newest to oldest


	.. py:attribute:: CreationTime

		Alias of :attr:`CreationDate <szurubooru_client.tokens.UserSortToken.CreationDate>`


	.. py:attribute:: LastLoginDate

		Recently active first


	.. py:attribute:: LastLoginTime

		Alias of :attr:`LastLoginDate <szurubooru_client.tokens.UserSortToken.CreationDate>`


	.. py:attribute:: LoginDate

		Alias of :attr:`LastLoginDate <szurubooru_client.tokens.UserSortToken.CreationDate>`


	.. py:attribute:: LoginTime

		Alias of :attr:`LastLoginDate <szurubooru_client.tokens.UserSortToken.CreationDate>`

.. _special-tokens-enum:

-------------------
Special Token Enums
-------------------

.. py:class:: szurubooru_client.tokens.PostSpecialToken


	.. py:attribute:: Liked

		Posts liked by currently logged-in user


	.. py:attribute:: Disliked

		Posts disliked by currently logged in user


	.. py:attribute:: Fav

		Posts added to favorites by currently logged-in user


	.. py:attribute:: Tumbleweed

		Posts with score of 0, without comments and without favorites
