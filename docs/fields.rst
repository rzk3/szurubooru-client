Field selection
===============

The Szurubooru API supports *Field Selection*. This allows you to select only a subset of fields to return from the API. By default all fields are selected and returned, but if you
pass in a list of field names to any of the client methods that support it, only those fields will be populated in the models.

Example
-------

```python
tags = client.list_tags(fields=["version", "names", "category"])
assert tags.results[0].description is None
```

Because the ``description`` field was not specified, it's set to ``None``
