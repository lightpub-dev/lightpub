from django.shortcuts import render, redirect
from django.views import View
from api.serializers.user import login_and_generate_token, RegisterSerializer
from . import forms

AUTH_COOKIE_NAME = "auth_token"


# Create your views here.
class LoginView(View):
    def get(self, request):
        form = forms.LoginForm()
        return render(request, "web/login.html", {"form": form})

    def post(self, request):
        form = forms.LoginForm(request.POST)
        if form.is_valid():
            username = form.cleaned_data["username"]
            password = form.cleaned_data["password"]
            token = login_and_generate_token(username, password)
            if token is None:
                # empty cookie
                response = render(
                    request,
                    "web/login.html",
                    {
                        "form": form,
                        "error": "Invalid username or password",
                    },
                )
                response.delete_cookie(AUTH_COOKIE_NAME)
                return response
            response = redirect("web:timeline")
            response.set_cookie(AUTH_COOKIE_NAME, token)
            return response
        return render(request, "web/login.html", {"form": form})


class RegisterView(View):
    def get(self, request):
        form = forms.RegisterForm()
        return render(request, "web/register.html", {"form": form})

    def post(self, request):
        form = forms.RegisterForm(request.POST)
        if form.is_valid():
            # create user
            ser = RegisterSerializer(data=form.cleaned_data)
            if not ser.is_valid():
                return render(
                    request,
                    "web/register.html",
                    {"form": form, "errors": ser.error_messages},
                )
            ser.save()
            return redirect("web:login")
        return render(request, "web/register.html", {"form": form})


class TimelineView(View):
    def get(self, request):
        return render(request, "web/timeline.html")
