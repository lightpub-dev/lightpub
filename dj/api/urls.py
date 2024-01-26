from django.urls import path, include

from .views import users, posts, browsable_login, interaction
from rest_framework.routers import DefaultRouter
from .views.posts import PostViewSet
from .views.interaction import PostFavoriteView, PostBookmarkView

app_name = "api"

router = DefaultRouter()
router.register(r"posts", PostViewSet, basename="post")
router.register(r"favorites", PostFavoriteView, basename="favorite")
router.register(r"bookmarks", PostBookmarkView, basename="bookmark")

urlpatterns = [
    path("register", users.RegisterView.as_view(), name="register"),
    path("login", users.LoginView.as_view(), name="login"),
    path("cookie-login", browsable_login.cookie_login, name="cookie-login"),
    path(
        "favorites/by-post-id/<uuid:post_id>",
        interaction.PostFavoriteByPostId.as_view(),
        name="favorite-by-post-id",
    ),
]

urlpatterns += router.urls

urlpatterns += [
    path("api-auth/", include("rest_framework.urls")),
]
