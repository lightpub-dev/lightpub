# Generated by Django 5.0.1 on 2024-01-26 13:11

import django.db.models.deletion
from django.db import migrations, models


class Migration(migrations.Migration):
    dependencies = [
        ("api", "0002_alter_user_username"),
    ]

    operations = [
        migrations.AlterField(
            model_name="post",
            name="poster",
            field=models.ForeignKey(
                on_delete=django.db.models.deletion.CASCADE,
                related_name="posts",
                to="api.user",
            ),
        ),
    ]
