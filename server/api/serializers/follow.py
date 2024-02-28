from rest_framework import serializers

from api import tasks
from api.models import User, UserFollow, UserFollowRequest
from api.utils.users import UserSpecifier, UserSpecifierSerializer
from lightpub.settings import HOSTNAME


class CreateFollowSerializer(serializers.Serializer):
    user_spec = UserSpecifierSerializer(write_only=True)
    message = serializers.CharField(read_only=True)

    def create(self, validated_data):
        user_spec: UserSpecifier = validated_data["user_spec"]
        user = user_spec.get_user_model()
        if user is None:
            # check if the user_spec is a remote user
            if user_spec.user_id:
                # if specified with user_id,
                # it is not a remote user unless it is stored in the db
                raise ValueError("user not found")

            if user_spec.username_and_host:
                # check if host specifies a remote host
                host = user_spec.username_and_host[1]
                if host is None:
                    # if host is not specified,
                    # it is not a remote user
                    raise ValueError("user not found")
                if host == HOSTNAME:
                    # if host is the same as this host,
                    # it is not a remote user
                    raise ValueError("user not found")

                # if host is different from this host,
                # it is a remote user
                # try to fetch the user from the remote host
                # TODO: prioritize task
                new_remote_user_id = tasks.fetch_remote_username.delay(
                    user_spec.username_and_host[0], host
                )
                new_remote_user = User.objects.get(id=new_remote_user_id.get())
                target_user = new_remote_user
                target_user_is_remote = True
            else:
                raise Exception("unreachable")
        else:
            target_user = user
            target_user_is_remote = user.host is not None

        following = self.context["request"].user
        if following == target_user:
            raise ValueError("cannot follow self")

        if not target_user_is_remote:
            UserFollow.objects.create(
                follower=following,
                followee=target_user,
            )
            return {"message": "followed"}
        else:
            fr = UserFollowRequest.objects.create(
                follower=following,
                followee=target_user,
            )

            tasks.send_follow_request.delay(fr.id)

            return {"message": "follow request sent"}


class FollowSerializer(serializers.ModelSerializer):
    class Meta:
        model = UserFollow
        fields = ["follower", "followee", "created_at"]
