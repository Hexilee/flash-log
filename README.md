# flash-log

A file-based logger designed for large throughputs.

## Benchmarks

> make bench

### SSD (~1GiB/s)

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

### HDD (~80MiB/s)

```log
running 1 test
write 1.0 KB in 664.807µs, avg latency: 583.676µs. 50%(583.676µs), 90%(583.676µs), 95%(583.676µs), 99%(583.676µs)
write 10.0 KB in 2.541953ms, avg latency: 2.241611ms. 50%(2.416371ms), 90%(2.416687ms), 95%(2.416687ms), 99%(2.416687ms)
write 100.0 KB in 4.35786ms, avg latency: 3.415524ms. 50%(3.196409ms), 90%(4.207333ms), 95%(4.208584ms), 99%(4.209015ms)
write 1000.0 KB in 14.955566ms, avg latency: 10.321395ms. 50%(9.769522ms), 90%(14.570896ms), 95%(14.62856ms), 99%(14.663165ms)
write 10.0 MB in 81.033421ms, avg latency: 43.255086ms. 50%(44.586673ms), 90%(71.556125ms), 95%(71.697084ms), 99%(71.926862ms)
write 100.0 MB in 1.128412082s, avg latency: 560.37788ms. 50%(550.703778ms), 90%(996.703703ms), 95%(1.085464633s), 99%(1.090011378s)
write 1000.0 MB in 8.150967169s, avg latency: 4.688047606s. 50%(4.976272875s), 90%(7.626309354s), 95%(7.633459285s), 99%(7.891117365s)
test test::test_write_data ... ok
```