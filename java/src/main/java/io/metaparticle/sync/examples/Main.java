package io.metaparticle.containerlib.elector.examples;

import io.metaparticle.sync.Lock;

public class Main {
    public static void main(String[] args) throws InterruptedException {
        Lock l;
        if (args.length > 0) {
            l = new Lock("test", args[0]);
        } else {
            l = new Lock("test");
        }
        System.out.println("Locking.");
        l.lock();
        System.out.println("Sleeping.");        
        Thread.sleep(60 * 1000);
        System.out.println("Unlocking.");
        l.unlock();
    }
}