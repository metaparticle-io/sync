package mpsync

import (
	"context"
)

// NewElection - return a new election.
func NewElection(locker Locker, leaderFn WorkFn, followerFn WorkFn) Election {
	return Election{
		locker:     locker,
		leaderFn:   leaderFn,
		followerFn: followerFn,
	}
}

// Election - type representing a metaparticle-sync election.
type Election struct {
	locker     Locker
	leaderFn   WorkFn
	followerFn WorkFn
}

// Run - Run the election.
func (e Election) Run(ctx context.Context) error {
	if err := e.locker.Lock(ctx, e.leaderFn); err != nil {
		return e.followerFn(ctx)
	}
	return nil
}
