from django import forms


class LoginForm(forms.Form):
    username = forms.CharField(label="Username", max_length=100)
    password = forms.CharField(label="Password", widget=forms.PasswordInput)


class RegisterForm(forms.Form):
    username = forms.CharField(
        label="Username", max_length=60, min_length=3, required=True
    )
    password = forms.CharField(
        label="Password", min_length=4, required=True, widget=forms.PasswordInput
    )
    nickname = forms.CharField(
        label="Nickname", max_length=200, min_length=3, required=True
    )
