namespace Metaparticle.Sync {
    public interface LockListener {
        void lockAcquired();
        void lockLost();
    }
}