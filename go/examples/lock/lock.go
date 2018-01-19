package main

import (
	"context"
	"fmt"
	"time"

	"github.com/metaparticle-io/sync/go/sync"
)

func onLock(c context.Context) error {
	fmt.Println("Got the lock!")
	time.Sleep(time.Second * 45)
	return nil
}

func main() {
	lock := mpsync.NewLock("just-a-lock", mpsync.DEFAULT_INTERVAL)
	if err := lock.Lock(context.Background(), onLock); err != nil {
		fmt.Printf("unexpected lock error: %s", err)
	}
}
