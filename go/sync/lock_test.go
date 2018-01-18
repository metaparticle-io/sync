package mpsync_test

import (
	"context"
	"net/url"
	"sync"
	"testing"
	"time"

	"github.com/metaparticle-io/sync/go/sync"
)

type MockLock struct {
	client   string
	deadline time.Time
}

type Server struct {
	locks sync.Map
}

type client struct {
	name   string
	server *Server
}

func newClient(name string, server *Server) client {
	return client{
		name:   name,
		server: server,
	}
}

func (c client) Get(url url.URL) (int, error) {
	_, ok := c.server.locks.Load(url.String())
	if !ok {
		return 404, nil
	}
	return 200, nil
}

func (c client) Put(url url.URL) (int, error) {
	value, ok := c.server.locks.Load(url.String())
	if ok {
		// Lock exists.
		// This lock isn't held by this client and the lock isn't up.
		lock := value.(MockLock)
		if lock.client != c.name && !time.Now().After(lock.deadline) {
			return 409, nil
		}
		// It's either OUR lock or we're after the deadline.
		c.server.locks.Delete(url.String())
	}

	_, loaded := c.server.locks.LoadOrStore(url.String(),
		MockLock{
			client:   c.name,
			deadline: time.Now().Add(time.Millisecond * 100),
		})
	if loaded {
		return 409, nil
	}

	return 200, nil
}

func TestLockingWithoutRetrying(t *testing.T) {
	workFn := func(_ context.Context) error {
		time.Sleep(time.Millisecond * 500)
		return nil
	}

	server := Server{}

	lock0 := mpsync.NewLock("super-important-lock", 100)
	lock1 := mpsync.NewLock("super-important-lock", 100)

	lock0.SetClient(newClient("client0", &server))
	lock1.SetClient(newClient("client1", &server))

	if err := lock0.Lock(context.Background(), workFn); err != nil {
		t.Errorf("workFn() failed unexpectedly! %+v", err)
	}

	if lock0.Locked() == false {
		t.Error("Expected lock0 to be locked!")
	}

	err := lock1.Lock(context.Background(), workFn)
	if err.Error() != "could not obtain a lock after 1 attempt(s)" {
		t.Errorf("unexpected error on failed lock: %s", err)
	}

	if lock1.Locked() == true {
		t.Error("Expected lock1 not to be locked!")
	}
}

func TestLockingWithRetries(t *testing.T) {
	workFn := func(_ context.Context) error {
		time.Sleep(time.Millisecond * 250)
		return nil
	}

	server := Server{}

	retryContext := mpsync.WithRetry(context.Background(), 5)

	lock0 := mpsync.NewLock("super-important-lock", 100)
	lock1 := mpsync.NewLock("super-important-lock", 100)

	lock0.SetClient(newClient("client0", &server))
	lock1.SetClient(newClient("client1", &server))

	if err := lock0.Lock(retryContext, workFn); err != nil {
		t.Errorf("workFn() failed unexpectedly! %+v", err)
	}

	if err := lock1.Lock(retryContext, workFn); err != nil {
		t.Errorf("workFn() failed unexpectedly! %+v", err)
	}

	if lock0.Locked() == false {
		t.Error("Expected lock0 to be locked!")
	}

	if lock1.Locked() == false {
		t.Error("Expected lock1 to be locked!")
	}
}

func TestLockingWithInfiniteRetries(t *testing.T) {
	workFn := func(_ context.Context) error {
		time.Sleep(time.Millisecond * 250)
		return nil
	}

	server := Server{}

	retryContext := mpsync.WithRetry(context.Background(), -1)

	lock0 := mpsync.NewLock("super-important-lock", 100)
	lock1 := mpsync.NewLock("super-important-lock", 100)

	lock0.SetClient(newClient("client0", &server))
	lock1.SetClient(newClient("client1", &server))

	if err := lock0.Lock(retryContext, workFn); err != nil {
		t.Errorf("workFn() failed unexpectedly! %+v", err)
	}

	if err := lock1.Lock(retryContext, workFn); err != nil {
		t.Errorf("workFn() failed unexpectedly! %+v", err)
	}

	if lock0.Locked() == false {
		t.Error("Expected lock0 to be locked!")
	}

	if lock1.Locked() == false {
		t.Error("Expected lock1 to be locked!")
	}
}
