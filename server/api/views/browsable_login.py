from django.http import HttpResponseRedirect
from django.shortcuts import render
from django.urls import reverse
from ..serializers.user import login_and_generate_token

from ..forms import LoginForm


def cookie_login(request):
    # if this is a POST request we need to process the form data
    if request.method == "POST":
        # create a form instance and populate it with data from the request:
        form = LoginForm(request.POST)
        # check whether it's valid:
        if form.is_valid():
            # process the data in form.cleaned_data as required
            username = form.cleaned_data.get("username")
            password = form.cleaned_data.get("password")

            token = login_and_generate_token(username, password)
            if token is None:
                return HttpResponseRedirect(reverse("api:cookie-login"))
            # set cookie
            response = HttpResponseRedirect("/api")
            response.set_cookie("auth_token", token)
            return response

    # if a GET (or any other method) we'll create a blank form
    else:
        form = LoginForm

    return render(request, "rest_framework/login.html", {"form": form})
