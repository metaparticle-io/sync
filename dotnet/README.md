# Examples MetaParticle Sync library for .NET Core

Metaparticle/Sync for .NET Core is a library that implements distributed synchronization
for cloud-native applications using a container side-car and Kubernetes primitives.

Metaparticle/Sync for .NET Core can be used for [locking](#locking-example) or for
[leader election](#election-example)

## Adding the Library
To add the `Metaparticle.Sync` library to your code you need to do two things:

   * Import the library, this is commonly done with: `using Metaparticle.Sync`
   * Run the `elector` side-car container. This is typically done via a Kubernetes Deployment (see examples below)

## Locking Example
The simplest usage is to deploy mutual exclusion locks between different distributed components

### Code
Here's the code for a simple locking example, that locks a lock named `test` and holds it for 45 seconds.

```cs
namespace LockExample
{
  using System;
  using System.Threading.Tasks;

  using Metaparticle.Sync;

  public class LockMain {
    public static void Main(string[] args) {
      Console.WriteLine("Locking");
      var l = new Lock("test");
      l.WaitOne();
      Console.WriteLine("Acquired lock, waiting for 45 seconds.");
      Task.Delay(45 * 1000).Wait();
      l.Release();
      Console.WriteLine("Lock released.");
    }
  }
}
```

You'll notice that a lock is made up of three things:
   * A name (this should be unique for a cluster)
   * A callback function to be called when the lock is acquired.
   * An optional callback function to be called when the lock is lost. If this is not supplied, the program will forcibly exit in the (unlikely) case that a lock is lost.

Simply creating a lock doesn't cause mutual exclusion. You also need to call `lock.lock()`. When
you are done, you call `lock.unlock()` to release the lock. Locks have a TTL (time to live) so
in the event of a failure, the lock will also be eventually lost.

### Deploying
To deploy code using the `Metaparticle.Sync` package, you need to also include a side-car that
does the heavy lifting for the lock. Your code and the sidecar should both be package as containers
and then deployed as a Kubernetes Pod.

Here is an example Kubernetes deployment:

```yaml
apiVersion: extensions/v1beta1
kind: Deployment
metadata:
  labels:
    run: lock-dotnet
  name: lock-dotnet
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      run: lock-dotnet
  template:
    metadata:
      labels:
        run: lock-dotnet 
    spec:
      containers:
      - image: brendanburns/elector
        name: elector
      - image: brendanburns/sync-dotnet
        name: example
```

You can create this with `kubectl create -f lock-deploy.yaml` which will create two different Pods, both of which are trying to obtain a lock named `test`.

## Election Example
An extension of locking is _leader election_ where a leader is chosen from a group of replicas.
This leader remains the leader for as long as it is healthy. If the leader ever fails, a new
leader is chosen. This is an extremely useful pattern for implementing a variety of distributed systems. Generally leader election is performed for a named _shard_ which represents some piece
of data to be owned/maintained by the leader.

### Code
Implementing leader election in `Metaparticle.Sync` is simple, here is code that performs
leader election for a shard named `test`.

```cs
namespace ElectionExample
{
  using System;
  using System.Threading;
  using System.Threading.Tasks;

  using Metaparticle.Sync;

  public class ElectionMain {
    public static void Main(string[] args) {
      var election = new Election(
        "test",
        () => {
          Console.WriteLine("I am the leader!");
        },
        () => {
          Console.WriteLine("I lost the leader!");
        });
      Console.WriteLine("Waiting for election");
      election.Run();
    }
  }
}
```

### Deploying leader election
As with locking, you need to deploy the elector side-car to take advantage of `Metaparticle.Sync` elections. Here's an example Kubernetes Deployment which deploys three leader replicas:

```yaml
apiVersion: extensions/v1beta1
kind: Deployment
metadata:
  labels:
    run: elector-dotnet
  name: elector-dotnet
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      run: elector-dotnet
  template:
    metadata:
      labels:
        run: elector-dotnet
    spec:
      containers:
      - image: brendanburns/elector
        imagePullPolicy: Always
        name: elector
        resources: {}
      # Replace the container below with your container.
      - image: brendanburns/dotnet-election
        name: example
```

## Technical Details
If you are interested in the technical details of how this all works, please see the [overview](../overview.md).