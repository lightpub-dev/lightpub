from django.urls import path

from . import views

app_name = "web"

urlpatterns = [
    path("login/", views.LoginView.as_view(), name="login"),
    path("register/", views.RegisterView.as_view(), name="register"),
    path("timeline/", views.TimelineView.as_view(), name="timeline"),
]
