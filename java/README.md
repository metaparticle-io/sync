# Examples MetaParticle Sync library for Java

## Adding the Library
To add the `io.metaparticle.sync` library to your code you need to do two things:

   * Import the library: `import io.metaparticle.sync.*;`
   * Run the `elector` side-car container. This is typically done via a Kubernetes Deployment (see examples below)

## Locking Example
The simplest usage is to deploy mutual exclusion locks between different distributed components

### Code
Here's the code for a simple locking example, that locks a lock named `test` and holds it for 45 seconds.

```java
import io.metaparticle.sync.Lock;

public class Main {
    public static void main(String[] args) throws InterruptedException {
        Lock lock = new Lock("test");
        System.out.println("Locking.");
        lock.lock();
        System.out.println("Sleeping.");        
        Thread.sleep(45 * 1000);
        System.out.println("Unlocking.");
        lock.unlock();
    }
}
```

You'll notice that a lock requires a name. This name should be unique for a cluster.

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
    run: lock-java
  name: lock-java
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      run: lock-java
  template:
    metadata:
      labels:
        run: lock-java 
    spec:
      containers:
      - image: brendanburns/elector
        name: elector
      - image: brendanburns/sync-java
        name: example
```

You can create this with `kubectl create -f lock-deploy.yaml` which will create two different Pods, both of which are trying to obtain a lock named `test`.

## Election Example
An extension of locking is _leader election_ where a leader is chosen from a group of replicas.
This leader remains the leader for as long as it is healthy. If the leader ever fails, a new
leader is chosen. This is an extremely useful pattern for implementing a variety of distributed systems. Generally leader election is performed for a named _shard_ which represents some piece
of data to be owned/maintained by the leader.

### Code
Implementing leader election in `io.metaparticle.sync` is simple, here is code that performs
leader election for a shard named `test`.

```java
import io.metaparticle.sync.Election;

public class ElectionMain {
    public static void main(String[] args) throws InterruptedException {
      Election e = new Election("test");
      e.addMasterListener(() -> {
        System.out.println("I am the master.");
        // <-- Do something as master here -->
      });
      e.addMasterLostListener(() -> {
        System.out.println("I lost the master.");
        // <-- Handle losing the master here -->
      });
}
```

### Deploying leader election
As with locking, you need to deploy the elector side-car to take advantage of `@metaparticle/sync` elections. Here's an example Kubernetes Deployment which deploys three leader replicas:

```yaml
apiVersion: extensions/v1beta1
kind: Deployment
metadata:
  labels:
    run: elector-java
  name: elector-java
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      run: elector-java
  template:
    metadata:
      labels:
        run: elector-java 
    spec:
      containers:
      - image: brendanburns/elector
        name: elector
      # Replace the container below with your container.
      - image: brendanburns/sync-java
        name: example
        command:
        - java
        - -classpath
        - /main.jar
        - io.metaparticle.sync.examples.ElectionMain
```

## Technical Details
If you are interested in the technical details of how this all works, please see the [overview](../overview.md).