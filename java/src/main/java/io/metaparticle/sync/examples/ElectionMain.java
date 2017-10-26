package io.metaparticle.sync.examples;

import io.metaparticle.sync.Election;

import java.util.Random;

public class ElectionMain {
    public static void main(String[] args) throws InterruptedException {
        Random r = new Random();
        while (true) {
            final Object block = new Object();
            Election e = new Election(
                "test", args[0],
                () -> {
                    System.out.println("I am the master.");
                    synchronized(block) {
                        try {
                            block.wait();
                        } catch (InterruptedException ex) {
                            ex.printStackTrace();
                        }
                    }
                },
                () -> {
                    System.out.println("I lost the master.");
                    synchronized(block) {
                        block.notify();
                    }
                });
            new Thread(e).start();
            Thread.sleep((r.nextInt(15) + 25) * 1000);
            e.shutdown();
            Thread.sleep(10 * 1000);
        }
    }
}