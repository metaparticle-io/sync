# Examples MetaParticle Sync library for Java

Metaparticle/Sync for Python is a library that implements distributed synchronization
for cloud-native applications using a container side-car and Kubernetes primitives.

Metaparticle/Sync for Python can be used for [locking](#locking-example) or for
[leader election](#election-example)

## Adding the Library
To add the `metaparticle_sync` library to your code you need to do two things:

   * Import the library: `import metaparticle_sync` 
   * Run the `elector` side-car container. This is typically done via a Kubernetes Deployment
   (see [examples](#deploying) below)

## Locking Example
The simplest usage is to deploy mutual exclusion locks between different distributed components

### Code
Here's the code for a simple locking example, that locks a lock named `test` and holds it for 45 seconds.

```python
from metaparticle_sync import Lock
import time

l = Lock('test')
l.acquire()
print('I have the lock!')
time.sleep(30)
l.release()
```

You'll notice that a lock requires a name. This name should be unique for a cluster.

Simply creating a lock doesn't cause mutual exclusion. You also need to call `lock.acquire()`. When
you are done, you call `lock.release() to release the lock. Locks have a TTL (time to live) so
in the event of a failure, the lock will also be eventually lost.


### Deploying
To deploy code using the `metaparticle_sync` package, you need to also include a side-car that
does the heavy lifting for the lock. Your code and the sidecar should both be package as containers
and then deployed as a Kubernetes Pod.

Here is an example Kubernetes deployment:

```yaml
apiVersion: extensions/v1beta1
kind: Deployment
metadata:
  labels:
    run: lock-python
  name: lock-python
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      run: lock-python
  template:
    metadata:
      labels:
        run: lock-python 
    spec:
      containers:
      - image: brendanburns/elector
        name: elector
      # Replace this with your image.
      - image: brendanburns/sync-python
        name: example
        env:
        - name: PYTHONUNBUFFERED
          value: "0"
        command:
        - python
        - -u
        - /lock.py
```

You can create this with `kubectl create -f lock-deploy.yaml` which will create two different Pods, both of which are trying to obtain a lock named `test`.

## Election Example
An extension of locking is _leader election_ where a leader is chosen from a group of replicas.
This leader remains the leader for as long as it is healthy. If the leader ever fails, a new
leader is chosen. This is an extremely useful pattern for implementing a variety of distributed systems. Generally leader election is performed for a named _shard_ which represents some piece
of data to be owned/maintained by the leader.

### Code
Implementing leader election in `metaparticle_sync` is simple, here is code that performs
leader election for a shard named `test`.

```python
import metaparticle_sync

def master_fn():
    print('I am the master')

def lost_master_fn():
    print('I lost the master')

el = metaparticle_sync.Election('test', master_fn, lost_master_fn)
el.run()

```

### Deploying leader election
As with locking, you need to deploy the elector side-car to take advantage of `io.metaparticle.sync` elections. Here's an example Kubernetes Deployment which deploys three leader replicas:

```yaml
apiVersion: extensions/v1beta1
kind: Deployment
metadata:
  labels:
    run: elector-python
  name: elector-python
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      run: elector-python
  template:
    metadata:
      labels:
        run: elector-python 
    spec:
      containers:
      - image: brendanburns/elector
        name: elector
      # Replace the container below with your container.
      - image: brendanburns/sync-python
        name: example
        env:
        - name: PYTHONUNBUFFERED
          value: "0"
        command:
        - python
        - -u
        - /election.py 
```

## Technical Details
If you are interested in the technical details of how this all works, please see the [overview](../overview.md).
