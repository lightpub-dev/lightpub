from django.urls import path

from . import views

app_name = "frontend"

urlpatterns = [
    path("", views.index, name="index"),
]
