package apub

import "context"

type DeliveryState struct {
	// requester
}

func NewDeliveryState() *DeliveryState {
	return &DeliveryState{}
}

func (s *DeliveryState) QueueUnsignedActivity(ctx context.Context, activity any, signer Actor, recipients []string) error {
	if s == nil {
		return nil
	}

	// TODO: sign payload and send to recipients
	return nil
}
