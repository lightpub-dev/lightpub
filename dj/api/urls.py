from django.urls import path

from . import views

APP_NAME = "api"

urlpatterns = [
    path("register", views.RegisterView.as_view(), name="register"),
    path("login", views.LoginView.as_view(), name="login"),
]
