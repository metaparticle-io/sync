package mpsync_test

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/metaparticle-io/sync/go/sync"
)

func TestElection(t *testing.T) {
	server := Server{}

	lock0 := mpsync.NewLock("super-important-lock", 100)
	lock1 := mpsync.NewLock("super-important-lock", 100)
	lock2 := mpsync.NewLock("super-important-lock", 100)

	lock0.SetClient(newClient("client0", &server))
	lock1.SetClient(newClient("client1", &server))
	lock2.SetClient(newClient("client2", &server))

	lChan := make(chan string, 1)
	fChan := make(chan string, 2)

	election0 := mpsync.NewElection(lock0,
		func(c context.Context) error {
			lChan <- "lock0"
			time.Sleep(time.Millisecond * 250)
			return nil
		},
		func(c context.Context) error {
			fChan <- "lock0"
			time.Sleep(time.Millisecond * 250)
			return nil
		})
	election1 := mpsync.NewElection(lock1,
		func(c context.Context) error {
			lChan <- "lock1"
			time.Sleep(time.Millisecond * 250)
			return nil
		},
		func(c context.Context) error {
			fChan <- "lock1"
			time.Sleep(time.Millisecond * 250)
			return nil
		})
	election2 := mpsync.NewElection(lock2,
		func(c context.Context) error {
			lChan <- "lock2"
			time.Sleep(time.Millisecond * 250)
			return nil
		},
		func(c context.Context) error {
			fChan <- "lock2"
			time.Sleep(time.Millisecond * 250)
			return nil
		})

	var wg sync.WaitGroup
	errors := make(chan error, 10)
	electionFn := func(e mpsync.Election) {
		defer wg.Done()
		errors <- e.Run(context.Background())
	}

	wg.Add(3)
	go electionFn(election0)
	go electionFn(election1)
	go electionFn(election2)

	wg.Wait()

	leader := <-lChan
	if leader != "lock2" {
		t.Errorf("expected lock2 to be leader, got %s instead", leader)
	}
}
