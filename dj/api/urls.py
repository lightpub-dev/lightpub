from django.urls import path, include

from .views import users, posts, browsable_login

app_name = "api"

urlpatterns = [
    path("register", users.RegisterView.as_view(), name="register"),
    path("login", users.LoginView.as_view(), name="login"),
    path("post", posts.CreatePostView.as_view(), name="post"),
    path("cookie-login", browsable_login.cookie_login, name="cookie-login"),
]

urlpatterns += [
    path("api-auth/", include("rest_framework.urls")),
]
