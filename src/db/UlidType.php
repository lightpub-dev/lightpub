<?php

namespace App\Doctrine\Type;

use Doctrine\DBAL\Platforms\AbstractPlatform;
use Doctrine\DBAL\Types\Type;
use Symfony\Component\Uid\Ulid;

/**
 * Custom Doctrine Type for mapping ULID to MySQL BINARY(16)
 */
class UlidType extends Type
{
    /**
     * The name of this type
     */
    public const NAME = 'ulid';

    /**
     * {@inheritdoc}
     */
    public function getName(): string
    {
        return self::NAME;
    }

    /**
     * {@inheritdoc}
     */
    public function getSQLDeclaration(array $column, AbstractPlatform $platform): string
    {
        return 'BINARY(16)';
    }

    /**
     * {@inheritdoc}
     * Converts a value from its PHP representation to its database representation of this type.
     */
    public function convertToDatabaseValue($value, AbstractPlatform $platform): ?string
    {
        if ($value === null) {
            return null;
        }

        if ($value instanceof Ulid) {
            return $value->toBinary();
        }

        if (is_string($value)) {
            return (new Ulid($value))->toBinary();
        }

        throw new \InvalidArgumentException('Invalid ULID value for database conversion');
    }

    /**
     * {@inheritdoc}
     * Converts a value from its database representation to its PHP representation of this type.
     */
    public function convertToPHPValue($value, AbstractPlatform $platform): ?Ulid
    {
        if ($value === null) {
            return null;
        }

        if (is_string($value)) {
            return Ulid::fromBinary($value);
        }

        throw new \InvalidArgumentException('Invalid database value for ULID conversion');
    }

    /**
     * {@inheritdoc}
     */
    public function requiresSQLCommentHint(AbstractPlatform $platform): bool
    {
        return true;
    }
}