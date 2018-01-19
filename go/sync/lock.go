package mpsync

import (
	"context"
	"fmt"
	"net/http"
	"net/url"
	"sync"
	"time"
)

const (
	// OK - Lock exists or we hold it.
	OK = 200

	// NOTFOUND - Lock does not exist.
	NOTFOUND = 404

	// CONFLICT - Lock is held by another caller.
	CONFLICT = 409

	// BaseLockURL - The URL of the sidecar container.
	BaseLockURL = "http://localhost:8080/"

	// DEFAULT_INTERVAL - default to 10 seconds interval between heartbeats.
	DEFAULT_INTERVAL = 10000
)

// WorkFn - A function representing a particle of work to be done when
//          the metaparticle-sync lock is taken.
type WorkFn func(context.Context) error

type retryKey string

// WithRetry - A context wrapped with a retry Value.
func WithRetry(ctx context.Context, retries int) context.Context {
	return context.WithValue(ctx, retryKey("retries"), retries)
}

// Locker - Interface that Locks.
type Locker interface {
	Lock(context.Context, WorkFn) error
}

// Lock -
type Lock struct {
	client   MockableClient
	isLocked bool
	interval int64
	lock     string
	lockURL  url.URL
}

// NewLock - Create a new lock.
func NewLock(lockStr string, interval int64) *Lock {
	baseURL, _ := url.Parse(BaseLockURL)
	lock, _ := url.Parse("locks/" + lockStr)
	lockURL := baseURL.ResolveReference(lock)

	return &Lock{
		client: httpClient{
			http.Client{
				Timeout: time.Second * 10,
			}},
		isLocked: false,
		interval: interval,
		lock:     lockStr,
		lockURL:  *lockURL,
	}
}

// Lock -
func (l *Lock) Lock(ctx context.Context, work WorkFn) error {
	if l.Locked() {
		return fmt.Errorf("locks are not reentrant")
	}
	code, err := l.client.Get(l.lockURL)
	if err != nil {
		return err
	}

	switch code {
	case OK, NOTFOUND:
		retries := ctx.Value(retryKey("retries"))
		if retries == nil {
			retries = 1
		}

		attempts := retries.(int)

		for {
			if attempts < 1 && attempts != -1 {
				return fmt.Errorf("could not obtain a lock after %d attempt(s)", retries)
			}

			code, err := l.client.Put(l.lockURL)
			if err != nil {
				// LOG
				return err
			}

			switch code {
			case OK:
				var wg sync.WaitGroup

				err := make(chan error, 1)
				heartbeatCtx, heartbeatCancel := context.WithCancel(ctx)
				workerCtx, workerCancel := context.WithCancel(ctx)

				heartbeat := func(c context.Context) {
					defer wg.Done()
					defer workerCancel()

					for {
						select {
						case <-c.Done():
							return
						default:
							time.Sleep(time.Millisecond * time.Duration(l.interval))
							code, err := l.client.Put(l.lockURL)
							if err != nil {
								// TODO - log
								return
							}
							switch code {
							case OK:
							case CONFLICT:
								// We lost the lock.
								return
							}
						}
					}
				}

				worker := func(c context.Context, e chan error) {
					defer wg.Done()
					defer heartbeatCancel()

					e <- work(c)
				}

				l.isLocked = true

				wg.Add(1)
				go heartbeat(heartbeatCtx)

				wg.Add(1)
				go worker(workerCtx, err)

				wg.Wait()

				return <-err
			case CONFLICT:
				// Try again
				if attempts != -1 {
					attempts--
				}
				time.Sleep(time.Millisecond * time.Duration(l.interval))
			default:
				return fmt.Errorf("Unexpected status: %d", code)
			}
		}
	default:
		return fmt.Errorf("Unexpected status: %d", code)
	}
}

// Locked - Return whether or not the lock was successfully taken.
func (l *Lock) Locked() bool {
	return l.isLocked
}

// SetClient - Helper for test mocks without having the tests be part of package.
func (l *Lock) SetClient(client MockableClient) {
	l.client = client
}

type heartbeat struct {
	interval uint
}
