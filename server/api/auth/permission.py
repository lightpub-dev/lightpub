from rest_framework.permissions import BasePermission, SAFE_METHODS
from ..models import (
    User,
    Post,
    UserFollow,
    PostFavorite,
    PostBookmark,
    PostAttachment,
    UploadedFile,
)


def _check_user_updatable(request, user, obj) -> bool:
    if isinstance(obj, User):
        print(f"{request.user} == {obj}")
        return request.user == obj

    if isinstance(obj, Post):
        return request.user == obj.poster

    if isinstance(obj, UserFollow):
        return request.user == obj.followee or request.user == obj.follower

    if isinstance(obj, PostFavorite):
        return request.user == obj.user

    if isinstance(obj, PostBookmark):
        return request.user == obj.user

    if isinstance(obj, UploadedFile):
        return request.user == obj.uploader

    if isinstance(obj, PostAttachment):
        return request.user == obj.post.poster

    return False


class AuthOnlyPermission(BasePermission):
    def has_permission(self, request, view):
        return isinstance(request.user, User)

    def has_object_permission(self, request, view, obj):
        if request.method in SAFE_METHODS:
            return True

        return _check_user_updatable(request, request.user, obj)


class NoAuthPermission(BasePermission):
    def has_permission(self, request, view):
        return True

    def has_object_permission(self, request, view, obj):
        if request.method in SAFE_METHODS:
            return True

        if isinstance(request.user, User):
            return _check_user_updatable(request, request.user, obj)

        return False


class NoPermission(BasePermission):
    def has_permission(self, request, view):
        return False

    def has_object_permission(self, request, view, obj):
        return False
