from django.urls import path, include

from .views import users, posts, browsable_login, interaction
from rest_framework.routers import DefaultRouter
from .views.posts import PostViewSet, UploadFileView
from .views.interaction import PostFavoriteView, PostBookmarkView
from .views.users import UserFollowingViewset, UserFollowerViewset, UserViewset
from .views.timeline import TimelineView
from .views.hashtags import PopularHashtagsView
from .views.pub import InboxView, OutboxView

app_name = "api"

router = DefaultRouter()
router.register(r"posts", PostViewSet, basename="post")
router.register(r"favorites", PostFavoriteView, basename="favorite")
router.register(r"bookmarks", PostBookmarkView, basename="bookmark")
router.register(r"followings", UserFollowingViewset, basename="following")
router.register(r"followers", UserFollowerViewset, basename="follower")
router.register(r"users", UserViewset, basename="user")
router.register(r"uploads", UploadFileView, basename="upload")
router.register(r"reactions", interaction.PostReactionView, basename="reaction")

urlpatterns = [
    path("register", users.RegisterView.as_view(), name="register"),
    path("login", users.LoginView.as_view(), name="login"),
    path("cookie-login", browsable_login.cookie_login, name="cookie-login"),
    path("timeline", TimelineView.as_view(), name="timeline"),
    path("trend/hashtags", PopularHashtagsView.as_view(), name="trend-hashtags"),
    path(
        "attachments/<uuid:pk>", posts.PostAttachmentView.as_view(), name="attachment"
    ),
    path(
        "user-avatars/<str:user_spec>",
        users.UserAvatarView.as_view(),
        name="user-avatar",
    ),
    path("replies/<uuid:pk>", posts.ReplyListView.as_view(), name="reply-list"),
    path("quotes/<uuid:pk>", posts.QuoteListView.as_view(), name="quote-list"),
    path("reposts/<uuid:pk>", posts.RepostListView.as_view(), name="repost-list"),
    path("favorites/<uuid:pk>", posts.FavoriteListView.as_view(), name="favorite-list"),
    path("inbox/<str:user_spec>", InboxView.as_view(), name="inbox"),
    path("outbox/<str:user_spec>", OutboxView.as_view(), name="outbox"),
]

urlpatterns += router.urls

urlpatterns += [
    path("api-auth/", include("rest_framework.urls")),
]
