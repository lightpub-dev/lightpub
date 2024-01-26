from django.urls import path, include

from .views import users, posts, browsable_login, interaction
from rest_framework.routers import DefaultRouter
from .views.posts import PostViewSet
from .views.interaction import PostFavoriteView, PostBookmarkView
from .views.users import UserFollowingViewset, UserFollowerViewset, UserViewset
from .views.timeline import TimelineView
from .views.hashtags import PopularHashtagsView

app_name = "api"

router = DefaultRouter()
router.register(r"posts", PostViewSet, basename="post")
router.register(r"favorites", PostFavoriteView, basename="favorite")
router.register(r"bookmarks", PostBookmarkView, basename="bookmark")
router.register(r"followings", UserFollowingViewset, basename="following")
router.register(r"followers", UserFollowerViewset, basename="follower")
router.register(r"users", UserViewset, basename="user")

urlpatterns = [
    path("register", users.RegisterView.as_view(), name="register"),
    path("login", users.LoginView.as_view(), name="login"),
    path("cookie-login", browsable_login.cookie_login, name="cookie-login"),
    path("timeline", TimelineView.as_view(), name="timeline"),
    path("trend/hashtags", PopularHashtagsView.as_view(), name="trend-hashtags"),
]

urlpatterns += router.urls

urlpatterns += [
    path("api-auth/", include("rest_framework.urls")),
]
