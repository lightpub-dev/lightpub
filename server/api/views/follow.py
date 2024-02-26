from rest_framework import generics
from rest_framework.response import Response

from api.auth.permission import AuthOnlyPermission
from api.serializers.follow import CreateFollowSerializer


class CreateFollowView(generics.CreateAPIView):
    serializer_class = CreateFollowSerializer
    permission_classes = [AuthOnlyPermission]

    def create(self, request, *args, **kwargs):
        ser = self.get_serializer(data=request.data)
        ser.is_valid(raise_exception=True)

        ser.save()

        return Response(status=201)
