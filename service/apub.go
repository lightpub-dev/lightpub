package service

import "context"

func (s *State) isAllowedToSend(ctx context.Context, targetURL string) bool {
	// TODO: localhost check
	// TODO: blocked instnace check
	return true
}
