from django.urls import include, path, register_converter
from rest_framework.routers import DefaultRouter

from .utils.users import UserSpecifierPath
from .views import browsable_login, follow, interaction, nodeinfo, posts, users
from .views.hashtags import PopularHashtagsView
from .views.posts import PostViewSet, UploadFileView
from .views.timeline import TimelineView
from .views.users import UserViewset

app_name = "api"

register_converter(UserSpecifierPath, "user_spec")

router = DefaultRouter(trailing_slash=False)
router.register(
    r"posts",
    PostViewSet,
    basename="post",
)
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
    path("follow", follow.CreateFollowView.as_view(), name="create-follow"),
    path(
        "follow/<user_spec:user>",
        follow.FollowView.as_view(),
        name="delete-follow",
    ),
    path("nodeinfo/2.0", nodeinfo.version_2_0, name="nodeinfo-2.0"),
    path("nodeinfo/2.1", nodeinfo.version_2_1, name="nodeinfo-2.1"),
]

urlpatterns += router.urls

urlpatterns += [
    path("api-auth/", include("rest_framework.urls")),
]
