# Generated by Django 5.0.1 on 2024-01-27 02:19

import uuid
from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ("api", "0004_uploadedfile_postattachment"),
    ]

    operations = [
        migrations.AlterField(
            model_name="postattachment",
            name="id",
            field=models.UUIDField(
                default=uuid.uuid4, primary_key=True, serialize=False
            ),
        ),
    ]
