from django.shortcuts import redirect

from lightpub.settings import FRONTEND_URL


# Create your views here.
def index(request):
    return redirect(FRONTEND_URL + request.path)
