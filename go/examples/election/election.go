package main

import (
	"context"
	"fmt"
	"time"

	"github.com/metaparticle-io/sync/go/sync"
)

func main() {
	lock := mpsync.NewLock("election-lock", mpsync.DEFAULT_INTERVAL)
	election := mpsync.NewElection(lock,
		func(c context.Context) error {
			fmt.Println("LEADER!")
			time.Sleep(time.Second * 45)
			return nil
		},
		func(c context.Context) error {
			fmt.Println("FOLLOWER")
			return nil
		})

	_ = election.Run(context.Background())
	time.Sleep(time.Second * 60)
}
