from rest_framework.pagination import CursorPagination


class MyPagination(CursorPagination):
    ordering = "-created_at"
