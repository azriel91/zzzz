using System;
using System.Threading;

public class Program {
    class Data {
        public int Value { get; set; }
    }

    public static void Main() {
Data data = new Data { Value = 0 };

Action inc0 = () => { for (int i = 0; i < 50000; i++) { data.Value += 1; } };
Action inc1 = () => { for (int i = 0; i < 50000; i++) { data.Value += 1; } };

Thread thread0 = new Thread(new ThreadStart(inc0));
Thread thread1 = new Thread(new ThreadStart(inc1));

thread0.Start();
thread1.Start();

thread0.Join();
thread1.Join();

Console.WriteLine($"value: {data.Value}");
    }
}
