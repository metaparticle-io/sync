namespace Metaparticle.Sync {
    using System;
    using System.Diagnostics;
    using System.Net;
    using System.Net.Http;
    using System.Net.Http.Headers;
    using System.Threading.Tasks;

    public class Lock {
        private Task Maintainer;
        private string Name;
        private bool Running;
        private string BaseUri;
        public LockListener Listener { set; get; }

        private static long WAIT_INTERVAL = 10 * 1000;

        public Lock(string name) : this(name, "http://localhost:13131") {}

        public Lock(string name, string baseUri) {
            this.Name = name;
            this.BaseUri = baseUri;
            this.Listener = null;
        }

        public void WaitOne() {
            while (!LockInternal(true, -1).Result);
        }

        public bool TryWait() {
            return LockInternal(false, 0).Result;
        }

    
        private bool AcquireLock() {
            HttpStatusCode code = HttpStatusCode.Unused;
            code = GetLock(Name);
            if (code == HttpStatusCode.NotFound || code == HttpStatusCode.OK) {
                code = UpdateLock(Name);
            }
            if (code == HttpStatusCode.OK) {
                HoldLock(Name);
                return true;
            }
            return false;
        }

        private async Task<bool> LockInternal(bool retry, long timeoutMillis) {
            Stopwatch watch = new Stopwatch();
            watch.Start();   
            do {
                long sleep = WAIT_INTERVAL;
                lock(this) {
                    if (Maintainer != null) {
                        throw new InvalidOperationException("Locks are not re-entrant!");
                    }
                    if (AcquireLock()) {
                        return true;
                    }
                    if (retry) {
                        sleep = timeoutMillis - watch.ElapsedMilliseconds;
                        if (timeoutMillis == -1 || sleep > WAIT_INTERVAL) {
                            sleep = WAIT_INTERVAL;
                        }
                    } else {
                        return false;
                    }
                }
                await Task.Delay((int) sleep);
            } while (timeoutMillis == -1 || watch.ElapsedMilliseconds < timeoutMillis);
            return false;
        }

        public void Release() {
            lock(this) {
                if (Maintainer == null) {
                    throw new InvalidOperationException("Lock is not held.");
                }
                Running = false;
                Maintainer.Wait(10 * 1000);
            } 
        }

        private HttpStatusCode GetLock(string name) {
            using (var client = new HttpClient()) {
                client.DefaultRequestHeaders.Accept.Clear();
                client.DefaultRequestHeaders.Accept.Add(
                    new MediaTypeWithQualityHeaderValue("application/json"));
                client.DefaultRequestHeaders.Add("User-Agent", "Metaparticle Sync Client");
                var result = client.GetAsync(BaseUri + "/locks/" + name).Result;
                return result.StatusCode;
            }
        }

        private HttpStatusCode UpdateLock(string name) {
            using (var client = new HttpClient()) {
                client.DefaultRequestHeaders.Accept.Clear();
                client.DefaultRequestHeaders.Accept.Add(
                    new MediaTypeWithQualityHeaderValue("application/json"));
                client.DefaultRequestHeaders.Add("User-Agent", "Metaparticle Sync Client");
                var result = client.PutAsync(BaseUri + "/locks/" + name, null).Result;
                return result.StatusCode;
            }
        }

        private void HoldLock(string name) {
            Running = true;
            if (Listener != null) {
                Listener.LockAcquired();
            }
            Maintainer = Task.Run(async () => {
                while (Running) {
                    HttpStatusCode code = GetLock(name);
                    if (code == HttpStatusCode.OK) {
                        code = UpdateLock(name);
                    }
                    if (code != HttpStatusCode.OK) {
                        Console.WriteLine("Unexpected status: " + code);
                        if (Listener != null) {
                            Listener.LockLost();
                            return;
                        } else {
//                                    Environment.Exit(0);
                        }
                    }
                    await Task.Delay(10 * 1000);
                }
                Maintainer = null;
                if (Listener != null) {
                    Listener.LockLost();
                }
            });
        }
    }
}