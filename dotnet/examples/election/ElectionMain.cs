namespace ElectionExample
{
  using System;
  using System.Threading;
  using System.Threading.Tasks;

  using Metaparticle.Sync;

  public class ElectionMain {
    public static void Main(string[] args) {
      var block = new Object();
      var election = new Election(
        "test",
        () => {
          Console.WriteLine("I am the leader!");
          Task.Delay(5 * 1000).Wait();
          Monitor.Pulse(block);
        },
        () => {
          Console.WriteLine("I lost the leader!");
        });
      Task.Run(() => {
        election.Run();
      });
      Monitor.Wait(block);
      election.Shutdown();
      // Wait for a while to let someone else win the election.
      Task.Delay(new Random().Next(40, 60) * 1000).Wait();
    }
  }
}