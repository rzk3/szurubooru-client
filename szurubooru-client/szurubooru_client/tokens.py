from .szurubooru_client import _tokens

anonymous_token = _tokens.anonymous_token
named_token = _tokens.named_token
sort_token = _tokens.sort_token
special_token = _tokens.special_token
CommentNamedToken = _tokens.CommentNamedToken
CommentSortToken = _tokens.CommentSortToken
PoolNamedToken = _tokens.PoolNamedToken
PoolSortToken = _tokens.PoolSortToken
PostNamedToken = _tokens.PostNamedToken
PostSortToken = _tokens.PostSortToken
PostSpecialToken = _tokens.PostSpecialToken
QueryToken = _tokens.QueryToken
SnapshotNamedToken = _tokens.SnapshotNamedToken
TagNamedToken = _tokens.TagNamedToken
TagSortToken = _tokens.TagSortToken
UserNamedToken = _tokens.UserNamedToken
UserSortToken = _tokens.UserSortToken

__doc__ = _tokens.__doc__
#if hasattr(_tokens, "__all__"):
#    __all__ = getattr(_tokens, "__all__")
