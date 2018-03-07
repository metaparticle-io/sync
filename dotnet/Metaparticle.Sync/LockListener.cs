namespace Metaparticle.Sync {
    public interface LockListener {
        void LockAcquired();
        void LockLost();
    }
}