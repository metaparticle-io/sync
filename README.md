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
      * [java](java)
      * [C#/dotnet core](dotnet)

## Examples
Examples for locking and leader election for each of the supported languages
can be found in their respective directories.
   * [javascript](javascript/README.md)
   * [java](java/README.md)
   * [C#/dotnet core](dotnet/README.md)

## Details
More technical details can be found in the [overview](overview.md).

## Community
   * Please file [isues or feature requests](https://github.com/metaparticle-io/sync/issues)
   * Please send Pull Requests!
   * Please visit us on [Slack](https://slack.metaparticle.io)
