# Metaparticle Sync libraries. 

## About the library 
Metaparticle is a standard library for cloud native development using containers and Kubernetes.

The Metaparticle *sync* is a library within Metaparticle for synchronization across multiple containers
running on different machines.

## Synchronization Primitives
Currently the library supports two synchronization primitives:
   * Locks, for mutual exclusion of code between different containers
   * Leader Election, for reliably selecting a single leader from a group of candidates, and failing over if that leader fails.

## Components
Metaparticle sync is made up of two components:
   * A re-usable container that can be deployed as a side-car to implement synchronization operators
   * A collection of idiomatic client libraries that users can use in their applications to
     implement synchronization.  Currently languages supported include:
      * [javascript (Nodejs)](javascript)
      * [java](java)
      * [C#/dotnet core](dotnet)


