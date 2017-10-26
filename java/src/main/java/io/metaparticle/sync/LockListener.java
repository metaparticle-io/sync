package io.metaparticle.sync;

public interface LockListener {
    public void lockAcquired();
    public void lockLost();
}