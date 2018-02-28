namespace Metaparticle.Sync {
    using System;
    using System.Threading;
    using System.Threading.Tasks;
    
    public class Election : LockListener {
        private Lock Lock;
        private Action ElectedAction;
        private Action TerminateAction;
        // End signal is used to verify that the electedAction terminates properly when terminateAction
        // is called.
        private Object EndSignal;
        private bool Done;
        private bool Running;
    
        public Election(string name, Action electedAction, Action terminateAction) :
            this(name, "http://localhost:13131", electedAction, terminateAction) {}

        public Election(string name, string baseUrl, Action electedAction, Action terminateAction) {
            this.Lock = new Lock(name, baseUrl);
            this.Lock.Listener = this;
            this.ElectedAction = electedAction;
            this.TerminateAction = terminateAction;
            this.EndSignal = new Object();
        }

        public void Shutdown() {
            Running = false;
            lock (Lock) {
                Monitor.Pulse(Lock);
            }
        }

        public void Run() {
            Running = true;
            while (Running) {
                Lock.WaitOne();
                lock (Lock) {
                    Monitor.Wait(Lock);
                }
            }
            Lock.Release();
        }

        public void LockAcquired() {
            Task.Run(() => {
                Done = false;
                ElectedAction();
                lock (EndSignal) {
                    Monitor.Pulse(EndSignal);
                    Done = true;
                }
            });
        }

        public void LockLost() {
            lock (EndSignal) {
                TerminateAction();
                // TODO: make this configurable?
                Task.Delay(1000).Wait();
                if (!Done) {
                    Console.WriteLine("Master didn't terminate in expected time, force terminating.");
                    //Application.exit(1);
                }
            }
            lock (Lock) {
                Monitor.Pulse(Lock);
            }
        }
    }
}