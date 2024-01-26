from rest_framework.permissions import BasePermission
from ..models import User


class AuthOnlyPermission(BasePermission):
    def has_permission(self, request, view):
        return isinstance(request.user, User)


class NoAuthPermission(BasePermission):
    def has_permission(self, request, view):
        return True


class NoPermission(BasePermission):
    def has_permission(self, request, view):
        return False
