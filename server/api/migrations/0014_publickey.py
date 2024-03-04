# Generated by Django 5.0.2 on 2024-02-23 14:50

import django.db.models.deletion
from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('api', '0013_user_private_key_user_public_key'),
    ]

    operations = [
        migrations.CreateModel(
            name='PublicKey',
            fields=[
                ('id', models.AutoField(primary_key=True, serialize=False)),
                ('uri', models.CharField(max_length=512)),
                ('public_key_pem', models.TextField(null=True)),
                ('last_fetched_at', models.DateTimeField(auto_now=True)),
                ('user', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, related_name='public_keys', to='api.user')),
            ],
        ),
    ]