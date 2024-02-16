from typing import Mapping, OrderedDict
from rest_framework import serializers
from rest_framework.settings import api_settings
from rest_framework.exceptions import ValidationError
from rest_framework.fields import SkipField, get_error_detail, set_value
from django.core.exceptions import ValidationError as DjangoValidationError
import enum

from api.requester import get_requester


class ActivityType(enum.StrEnum):
    FOLLOW = "https://www.w3.org/ns/activitystreams#Follow"
    UNDO = "https://www.w3.org/ns/activitystreams#Undo"


class ActivityPubSerializer(serializers.Serializer):
    def to_internal_value(self, data):
        """
        Dict of native values <- Dict of primitive datatypes.
        """
        if not isinstance(data, Mapping):
            message = self.error_messages["invalid"].format(
                datatype=type(data).__name__
            )
            raise ValidationError(
                {api_settings.NON_FIELD_ERRORS_KEY: [message]},
                code="invalid",
            )

        ret = OrderedDict()
        errors = OrderedDict()
        fields = self._writable_fields

        for field in fields:
            validate_method = getattr(self, "validate_" + field.field_name, None)
            if not isinstance(field, AccessField):
                primitive_value = field.get_value(data)
            else:
                primitive_value = data
                for p in field.path:
                    primitive_value = primitive_value[p]
            try:
                validated_value = field.run_validation(primitive_value)
                if validate_method is not None:
                    validated_value = validate_method(validated_value)
            except ValidationError as exc:
                errors[field.field_name] = exc.detail
            except DjangoValidationError as exc:
                errors[field.field_name] = get_error_detail(exc)
            except SkipField:
                pass
            else:
                if not isinstance(field, AccessField):
                    set_value(ret, field.source_attrs, validated_value)
                else:
                    set_value(ret, [field.field_name], validated_value)

        if errors:
            raise ValidationError(errors)

        return ret


ACTOR_KEY = "https://www.w3.org/ns/activitystreams#actor"
OBJECT_KEY = "https://www.w3.org/ns/activitystreams#object"


class AccessField(serializers.Field):
    def __init__(
        self, serializer: serializers.Field, path: list[str] = [], *args, **kwargs
    ):
        if not serializer:
            raise ValueError("serializer is required")
        self._ser = serializer
        if isinstance(path, list):
            if len(path) == 0:
                raise ValueError("path must be a non-empty list")
        elif isinstance(path, str):
            path = [path]
        self.path = path
        super().__init__(*args, **kwargs)

    def to_internal_value(self, data):
        return self._ser.to_internal_value(data)


class SingleArrayField(serializers.Serializer):
    def __init__(self, *args, **kwargs):
        if "many" in kwargs:
            raise ValueError("many kw is not supported")
        ser = kwargs.pop("serializer")
        kwargs["many"] = True
        super().__init__(*args, **kwargs)

        self._ser = ser

    def to_internal_value(self, data):
        return self._ser.to_internal_value(data[0])

    def to_representation(self, value):
        return [self._ser.to_representation(value)]


class IdentifiableSerializer(ActivityPubSerializer):
    id = serializers.URLField()

    def get_fields(self):
        fields = super().get_fields()
        fields["@id"] = fields.pop("id")
        return fields


class ActivitySerializer(IdentifiableSerializer):
    type = SingleArrayField(serializer=serializers.CharField())

    def get_fields(self):
        fields = super().get_fields()
        fields["@type"] = fields.pop("type")
        return fields


class FollowSerializer(ActivitySerializer):
    actor = AccessField(
        serializer=SingleArrayField(serializer=IdentifiableSerializer()),
        path=[ACTOR_KEY],
    )
    object = AccessField(
        serializer=SingleArrayField(serializer=IdentifiableSerializer()),
        path=[OBJECT_KEY],
    )

    def to_internal_value(self, data):
        result = super().to_internal_value(data)
        return result


class NetworkFollowSerializer(ActivitySerializer):
    def to_internal_value(self, data):
        # if "@id" is the only key, then need to fetch the remote object
        if len(data) == 1 and "@id" in data:
            req = get_requester()
            expanded = req.fetch_remote_id(data)
            ser = FollowSerializer()
            return ser.to_internal_value(expanded[0])

        return FollowSerializer().to_internal_value(data)


class UndoSerializer(ActivitySerializer):
    object = AccessField(
        serializer=SingleArrayField(serializer=NetworkFollowSerializer()),
        path=[OBJECT_KEY],
    )

    def to_internal_value(self, data):
        result = super().to_internal_value(data)
        return result


serializer_map = {"https://www.w3.org/ns/activitystreams#Follow": FollowSerializer}
