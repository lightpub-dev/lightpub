# Generated by Django 5.0.2 on 2024-02-25 07:32

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ("api", "0016_federatedserver"),
    ]

    operations = [
        migrations.AddField(
            model_name="post",
            name="partial",
            field=models.BooleanField(default=False),
        ),
    ]