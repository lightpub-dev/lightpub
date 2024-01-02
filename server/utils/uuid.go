package utils

import "github.com/google/uuid"

func GenerateUUIDString() (string, error) {
	uuid, err := uuid.NewRandom()
	if err != nil {
		return "", err
	}
	return uuid.String(), nil
}

func GenerateUUIDBytes() ([]byte, error) {
	uuid, err := uuid.NewRandom()
	if err != nil {
		return nil, err
	}
	uuidBytes, err := uuid.MarshalBinary()
	return uuidBytes, err
}

func GenerateUUIDBoth() (string, []byte, error) {
	uuid, err := uuid.NewRandom()
	if err != nil {
		return "", nil, err
	}
	uuidBytes, err := uuid.MarshalBinary()
	return uuid.String(), uuidBytes, err
}
