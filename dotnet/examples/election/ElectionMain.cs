namespace ElectionExample
{
  using System;
  using System.Threading;
  using System.Threading.Tasks;

  using Metaparticle.Sync;

  public class ElectionMain {
    public static void Main(string[] args) {
      var election = new Election(
        "test",
        () => {
          Console.WriteLine("I am the leader!");
        },
        () => {
          Console.WriteLine("I lost the leader!");
        });
      Console.WriteLine("Waiting for election");
      election.Run();
    }
  }
}