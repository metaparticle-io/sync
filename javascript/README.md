# Examples MetaParticle Sync library for Javascript

## Adding the Library
To add the `@metaparticle/sync` library to your code you need to do two things:

   * Import the library, this is commonly done with: `var mp = require('@metaparticle/sync');`
   * Run the `elector` side-car container. This is typically done via a Kubernetes Deployment (see examples below)

## Locking Example
The simplest usage is to deploy mutual exclusion locks between different distributed components

### Code
Here's the code for a simple locking example, that locks a lock named `test` and holds it for 45 seconds.

```javascript
// Import the library
var mp = require('@metaparticle/sync');

// Create a new lock.
var lock = new mp.Lock(
    // The name of the lock.
    'test',
    // This handler is called when the lock is acquired.
    () => {
       console.log('I have the lock!');
       console.log('Holding the lock for 45 seconds');
       setTimeout(() => {
           // Unlock after 45 seconds.
           lock.unlock();
           console.log('Unlocked');
       }, 45 * 1000);
    },
    // [optional] this handler is called when the lock is lost
    () => {
       console.log('I lost the lock!');
    });

// Kick off the lock, eventually this will call the callbacks above.
console.log('Attempting to lock');
lock.lock();
```

You'll notice that a lock is made up of three things:
   * A name (this should be unique for a cluster)
   * A callback function to be called when the lock is acquired.
   * An optional callback function to be called when the lock is lost. If this is not supplied, the program will forcibly exit in the (unlikely) case that a lock is lost.

Simply creating a lock doesn't cause mutual exclusion. You also need to call `lock.lock()`. When
you are done, you call `lock.unlock()` to release the lock. Locks have a TTL (time to live) so
in the event of a failure, the lock will also be eventually lost.

### Deploying
To deploy code using the `@metaparticle/sync` package, you need to also include a side-car that
does the heavy lifting for the lock. Your code and the sidecar should both be package as containers
and then deployed as a Kubernetes Pod.

Here is an example Kubernetes deployment:

```yaml
apiVersion: extensions/v1beta1
kind: Deployment
metadata:
  labels:
    run: lock-js
  name: lock-js
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      run: lock-js
  template:
    metadata:
      labels:
        run: lock-js 
    spec:
      containers:
      - image: brendanburns/elector
        name: elector
      - image: brendanburns/sync-js
        name: example
```

You can create this with `kubectl create -f lock-deploy.yaml` which will create two different Pods, both of which are trying to obtain a lock named `test`.

## Election Example
An extension of locking is _leader election_ where a leader is chosen from a group of replicas.
This leader remains the leader for as long as it is healthy. If the leader ever fails, a new
leader is chosen. This is an extremely useful pattern for implementing a variety of distributed systems. Generally leader election is performed for a named _shard_ which represents some piece
of data to be owned/maintained by the leader.

### Code
Implementing leader election in `@metaparticle/sync` is simple, here is code that performs
leader election for a shard named `test`.

```javascript
var mp = require('@metaparticle/sync');

var election = new mp.Election(
    // Name of the election shard
    'test',
    // Event handler, called when a program becomes the leader.
    () => {
        console.log('I am the leader');
    },
    // Event handler, called when a program that was leader is no longer leader.
    () => {
        console.log('I lost the leader');
    });

election.run();
```

### Deploying leader election
As with locking, you need to deploy the elector side-car to take advantage of `@metaparticle/sync` elections. Here's an example Kubernetes Deployment which deploys three leader replicas:

```yaml
apiVersion: extensions/v1beta1
kind: Deployment
metadata:
  labels:
    run: elector-js
  name: elector-js
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      run: elector-js
  template:
    metadata:
      labels:
        run: elector-js 
    spec:
      containers:
      - image: brendanburns/elector
        imagePullPolicy: Always
        name: elector
        resources: {}
      # Replace the container below with your container.
      - image: brendanburns/sync-js
        name: example
        command:
        - node
        - elector.js
```

## Technical Details
If you are interested in the technical details of how this all works, please see the [overview](../overview.md).