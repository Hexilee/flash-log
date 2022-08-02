# flash-log

A file-based logger designed for large throughputs.

## Benchmarks

> make bench

### SSD (~1.1GiB/s)

```log
running 1 test
write 1.0 KB in 311.353µs, avg latency: 246.722µs. 50%(246.722µs), 90%(246.722µs), 95%(246.722µs), 99%(246.722µs)
write 10.0 KB in 559.253µs, avg latency: 360.129µs. 50%(470.466µs), 90%(472.546µs), 95%(472.546µs), 99%(472.546µs)
write 100.0 KB in 1.110952ms, avg latency: 900.03µs. 50%(945.643µs), 90%(1.028278ms), 95%(1.028963ms), 99%(1.029798ms)
write 1000.0 KB in 2.259327ms, avg latency: 1.376525ms. 50%(1.30648ms), 90%(1.97073ms), 95%(1.97355ms), 99%(1.977995ms)
write 10.0 MB in 13.922012ms, avg latency: 7.286059ms. 50%(7.221826ms), 90%(11.260955ms), 95%(11.282207ms), 99%(11.333073ms)
write 100.0 MB in 111.346703ms, avg latency: 49.800044ms. 50%(49.964804ms), 90%(78.374833ms), 95%(83.337817ms), 99%(83.590387ms)
write 1000.0 MB in 990.51046ms, avg latency: 378.676661ms. 50%(386.271167ms), 90%(623.035233ms), 95%(657.001782ms), 99%(674.123562ms)
test test::test_write_data ... ok
```

### HDD (~168MiB/s)

```log
running 1 test
write 1.0 KB in 670.81µs, avg latency: 557.942µs. 50%(557.942µs), 90%(557.942µs), 95%(557.942µs), 99%(557.942µs)
write 10.0 KB in 1.837829ms, avg latency: 1.36268ms. 50%(1.359492ms), 90%(1.739626ms), 95%(1.739626ms), 99%(1.739626ms)
write 100.0 KB in 4.88041ms, avg latency: 3.638901ms. 50%(3.443846ms), 90%(4.756498ms), 95%(4.756849ms), 99%(4.762367ms)
write 1000.0 KB in 15.457012ms, avg latency: 10.004802ms. 50%(9.777296ms), 90%(12.834927ms), 95%(15.083186ms), 99%(15.105382ms)
write 10.0 MB in 169.356069ms, avg latency: 62.491729ms. 50%(61.214729ms), 90%(102.534186ms), 95%(159.916845ms), 99%(163.656266ms)
write 100.0 MB in 1.403721103s, avg latency: 729.024237ms. 50%(749.763877ms), 90%(1.301503876s), 95%(1.30613071s), 99%(1.372349294s)
write 1000.0 MB in 7.176729815s, avg latency: 4.078706759s. 50%(4.235005525s), 90%(6.804786237s), 95%(6.813282432s), 99%(6.93267018s)
test test::test_write_data ... ok
```
