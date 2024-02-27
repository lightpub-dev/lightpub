from rest_framework import serializers


class LinkSerializer(serializers.Serializer):
    href = serializers.CharField(max_length=255)
    rel = serializers.CharField(max_length=255)
    type = serializers.CharField(max_length=255, required=False)


class UserSerializer(serializers.Serializer):
    aliases = serializers.ListField(child=serializers.URLField())
    links = LinkSerializer(many=True)
    subject = serializers.CharField(max_length=255)
