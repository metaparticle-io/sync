package io.metaparticle.sync;

public class Election implements LockListener, Runnable {
    private Lock lock;
    private Runnable electedAction;
    private Runnable terminateAction;
    // End signal is used to verify that the electedAction terminates properly when terminateAction
    // is called.
    private Object endSignal;
    private boolean done;
    private boolean running;
    
    public Election(String name, Runnable electedAction, Runnable terminateAction) {
        this(name, "http://localhost:13131", electedAction, terminateAction);
    }

    public Election(String name, String baseUrl, Runnable electedAction, Runnable terminateAction) {
        this.lock = new Lock(name, baseUrl);
        this.lock.setLockListener(this);
        this.electedAction = electedAction;
        this.terminateAction = terminateAction;
        this.endSignal = new Object();
    }

    public void shutdown() {
        running = false;
        synchronized (lock) {
            lock.notify();
        }
    }

    @Override
    public void run() {
        running = true;
        while (running) {
            try {
                lock.lock();
                synchronized (lock) {
                    lock.wait();
                }
            } catch (InterruptedException ex) {
                ex.printStackTrace();
            }
        }
        lock.unlock();
    }

    @Override
    public void lockAcquired() {
        new Thread(new Runnable() {
            public void run() {
                done = false;
                electedAction.run();
                synchronized (endSignal) {
                    endSignal.notify();
                    done = true;
                }
            }
        }).start();
    }

    @Override
    public void lockLost() {
        synchronized (endSignal) {
            terminateAction.run();
            // TODO: make this configurable?
            try {
                endSignal.wait(1000);
            } catch (InterruptedException ex) {
                ex.printStackTrace();
            }
            if (!done) {
                System.err.println("Master didn't terminate in expected time, force terminating.");
                System.exit(1);
            }
        }
        synchronized (lock) {
            lock.notify();
        }
    }

}