package io.metaparticle.sync;

import com.mashape.unirest.http.HttpResponse;
import com.mashape.unirest.http.JsonNode;
import com.mashape.unirest.http.Unirest;
import com.mashape.unirest.http.exceptions.UnirestException;
import java.io.IOException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.locks.Condition;

public class Lock implements java.util.concurrent.locks.Lock {
    // TODO: Switch to a threadpool here?
    private Thread maintainer;
    private String name;
    private boolean running;
    private String baseUri;
    private LockListener listener;

    private static long WAIT_INTERVAL = 10 * 1000;

    public Lock(String name) {
        this(name, "http://localhost:13131");
    }

    public Lock(String name, String baseUri) {
        this.name = name;
        this.baseUri = baseUri;
        this.listener = null;
    }

    public void setLockListener(LockListener l) {
        listener = l;
    }

    @Override
    public void lock() {
        while (true) {
            try {
                lockInterruptibly();
                return;
            } catch (InterruptedException ignore) {}
        }
    }

    @Override
    public void lockInterruptibly() throws InterruptedException {
        lockInternal(true, -1);
    }

    @Override
    public boolean tryLock() {
        try {
            return lockInternal(false, 0);
        } catch (InterruptedException ex) {
            // This can never actually happen on this code-path.
        }
        return false;
    }

    @Override
    public boolean tryLock(long time, TimeUnit unit) throws InterruptedException {
        return lockInternal(true, TimeUnit.MILLISECONDS.convert(time, unit));
    }

    private boolean acquireLock() {
        int code = -1;
        try {
            code = getLock(name);
            if (code == 404 || code == 200) {
                code = updateLock(name);
            }
            if (code == 200) {
                holdLock(name);
                return true;
            }
        } catch (IOException ex) {
            ex.printStackTrace();
        }
        return false;
    }

    private synchronized boolean lockInternal(boolean retry, long timeoutMillis) throws InterruptedException {
        if (maintainer != null) {
            throw new IllegalStateException("Locks are not re-entrant!");
        }
        long deadline = System.currentTimeMillis() + timeoutMillis;
        do {
            if (acquireLock()) {
                return true;
            }
            if (retry) {
                long sleep = deadline - System.currentTimeMillis();
                if (timeoutMillis == -1 || (deadline - System.currentTimeMillis()) > WAIT_INTERVAL) {
                    sleep = WAIT_INTERVAL;
                }
                Thread.sleep(sleep);
            } else {
                return false;
            }
        } while (timeoutMillis == -1 || System.currentTimeMillis() < deadline);
        return false;
    }

    @Override
    public Condition newCondition() {
        // TODO: try to support this?
        throw new UnsupportedOperationException("unsupported.");
    }

    @Override
    public synchronized void unlock() {
        if (maintainer == null) {
            throw new IllegalStateException("Lock is not held.");
        }
        running = false;
        try {
            maintainer.join(10 * 1000);
        } catch (InterruptedException ex) {
            ex.printStackTrace();
        }
    }

    private int getLock(String name) throws IOException {
        try {
            HttpResponse<JsonNode> jsonResponse = Unirest.get(baseUri + "/locks/" + name)
                .header("accept", "application/json")
                .asJson();
            return jsonResponse.getStatus();
        } catch (UnirestException ex) {
            throw new IOException(ex);
        }
    }

    private int updateLock(String name) throws IOException {
        try {
            HttpResponse<JsonNode> jsonResponse = Unirest.put(baseUri + "/locks/" + name)
                .header("accept", "application/json")
                .asJson();
            return jsonResponse.getStatus();
        } catch (UnirestException ex) {
            throw new IOException(ex);
        }
    }

    private void holdLock(final String name) {
        running = true;
        if (listener != null) {
            listener.lockAcquired();
        }
        maintainer = new Thread(new Runnable() {
            public void run() {
                while (running) {
                    try {
                        int code = getLock(name);
                        if (code == 200) {
                            code = updateLock(name);
                        }
                        if (code != 200) {
                            System.out.println("Unexpected status: " + code);
                            if (listener != null) {
                                listener.lockLost();
                                return;
                            } else {
                                System.exit(0);
                            }
                        }
                        Thread.sleep(10 * 1000);
                    } catch (IOException | InterruptedException ex) {
                        ex.printStackTrace();
                    }
                }
                maintainer = null;
                if (listener != null) {
                    listener.lockLost();
                }               
            }
        });
        maintainer.start();
    }
}