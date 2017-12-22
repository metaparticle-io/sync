# Metaparticle Sync libraries. 

## About the library 
Metaparticle is a standard library for cloud native development using containers and Kubernetes.

The Metaparticle *sync* is a library within Metaparticle for synchronization across multiple containers
running on different machines.

## Synchronization Primitives
Currently the library supports two synchronization primitives:
   * Locks, for mutual exclusion of code between different containers on different machines.
   * Leader Election, for reliably selecting a single leader from a group of candidates, and failing over if that leader fails.

## Components
Metaparticle sync is made up of two components:
   * A re-usable container that can be deployed as a side-car to implement synchronization operators
   * A collection of idiomatic client libraries that users can use in their applications to
     implement synchronization.  Currently languages supported include:
      * [javascript (Nodejs)](javascript)
      * [python](python)
      * [java](java)
      * [C#/.NET Core](dotnet)

## Examples
Examples for locking and leader election for each of the supported languages
can be found in their respective directories.
   * [javascript](javascript/README.md)
   * [python](python/README.md)
   * [java](java/README.md)
   * [C#/.NET Core](dotnet/README.md)

## Details
More technical details can be found in the [overview](overview.md).

## Contribute
There are many ways to contribute to Metaparticle

 * [Submit bugs](https://github.com/metaparticle-io/package/issues) and help us verify fixes as they are checked in.
 * Review the source code changes.
 * Engage with other Metaparticle users and developers on [gitter](https://gitter.im/metaparticle-io/Lobby).
 * Join the #metaparticle discussion on [Twitter](https://twitter.com/MetaparticleIO).
 * [Contribute bug fixes](https://github.com/metaparticle-io/package/pulls).

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto://opencode@microsoft.com) with any additional questions or comments.

