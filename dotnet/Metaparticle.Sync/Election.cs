namespace Metaparticle.Sync {
    using System;
    using System.Threading;
    using System.Threading.Tasks;
    
    public class Election : LockListener {
        private Lock lck;
        private Action electedAction;
        private Action terminateAction;
        // End signal is used to verify that the electedAction terminates properly when terminateAction
        // is called.
        private Object endSignal;
        private bool done;
        private bool running;
    
        public Election(string name, Action electedAction, Action terminateAction) :
            this(name, "http://localhost:13131", electedAction, terminateAction) {}

        public Election(string name, string baseUrl, Action electedAction, Action terminateAction) {
            this.lck = new Lock(name, baseUrl);
            this.lck.Listener = this;
            this.electedAction = electedAction;
            this.terminateAction = terminateAction;
            this.endSignal = new Object();
        }

        public void shutdown() {
            running = false;
            lock (lck) {
                Monitor.Pulse(lck);
            }
        }

        public void run() {
            running = true;
            while (running) {
                lck.WaitOne();
                lock (lck) {
                    Monitor.Wait(lck);
                }
            }
            lck.Release();
        }

        public void lockAcquired() {
            Task.Run(() => {
                done = false;
                electedAction();
                lock (endSignal) {
                    Monitor.Pulse(endSignal);
                    done = true;
                }
            });
        }

        public void lockLost() {
            lock (endSignal) {
                terminateAction();
                // TODO: make this configurable?
                Task.Delay(1000).Wait();
                if (!done) {
                    Console.WriteLine("Master didn't terminate in expected time, force terminating.");
                    //Application.exit(1);
                }
            }
            lock (lck) {
                Monitor.Pulse(lck);
            }
        }
    }
}