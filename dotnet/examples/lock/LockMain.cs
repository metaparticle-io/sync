namespace LockExample
{
  using System;
  using System.Threading.Tasks;

  using Metaparticle.Sync;

  public class LockMain {
    public static void Main(string[] args) {
      Console.WriteLine("Locking");
      var l = new Lock("test");
      l.WaitOne();
      Console.WriteLine("Acquired lock, waiting for 45 seconds.");
      Task.Delay(45 * 1000).Wait();
      l.Release();
      Console.WriteLine("Lock released.");
    }
  }
}

