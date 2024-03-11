import os

from celery import Celery

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "lightpub.settings")

app = Celery("lightpub")

app.config_from_object("django.conf:settings", namespace="CELERY")
app.autodiscover_tasks()
