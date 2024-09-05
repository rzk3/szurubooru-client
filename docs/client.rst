Szurubooru Clients
==================

These are the clients available for interacting with Szurubooru. There are two clients, ``SzurubooruSyncClient`` and ``SzurubooruAsyncClient`` . The only difference
between the two of them is that ``SzurubooruAsyncClient`` is compatible with ``asyncio`` for use in ``async`` scripts.

.. contents:: Table of Contents
    :depth: 3
    :local:

Sync Client
-----------

.. autoclass:: szurubooru_client.SzurubooruSyncClient


Async Client
------------

.. autoclass:: szurubooru_client.SzurubooruAsyncClient


Exceptions
----------

All exceptions that are thrown by the client are instances of the ``SzuruClientError`` class. It's a tuple of two items: An exception type (as a string) and a string with more details
about the exception.

.. autoexception:: szurubooru_client.SzuruClientError

.. _rver:

Resource Versioning
-------------------

Many of the ``update`` and other methods on the client classes accept a ``version`` argument. This is part of Szurubooru's optimistic locking. Each resource has its ``version`` retuned
to the client through any of the ``get`` methods. This value must be provided to any of the ``update`` or ``delete`` methods. If the version doesn't match at the time
(due to a modified resource) then the call will fail.

Pagination
----------

Most of the ``list`` methods on the clients return a paged object that holds the results along with other details about the query:

.. autoclass:: szurubooru_client.PagedResult

Paging through the results of ``list`` methods are done using two parameters:

.. _limits:

Result Limits
^^^^^^^^^^^^^

Most of the ``list`` methods on the clients support a ``limit`` parameter. This parameter limits the number of resources returned by the method as part of
a :class:`~szurubooru_client.PagedResult` object

.. _offsets:

Result Offsets
^^^^^^^^^^^^^^

Most of the ``list`` methods on the clients support a ``offset`` parameter. This parameter skips through the results returned by the method as part of
a :class:`~szurubooru_client.PagedResult` object
