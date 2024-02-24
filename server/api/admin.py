from django.contrib import admin

from . import models

admin.site.register(models.User)
admin.site.register(models.UserProfileLabel)
admin.site.register(models.UserFollow)
admin.site.register(models.UserToken)
admin.site.register(models.Post)
admin.site.register(models.PostHashtag)
admin.site.register(models.PostFavorite)
admin.site.register(models.PostBookmark)
admin.site.register(models.PostMention)
admin.site.register(models.PublicKey)
admin.site.register(models.UserFollowRequest)
