# ben
A command-line benchmarking tool with resource usage monitoring.

## Usage
```
ben "xxx -a -b -c cc.cc"
```
Will write `ben.log` file with time vs memory information.

```
ben "xxx -a -b -c cc.cc" -o output.log
```
Add `-o` to write result to a specific directory.

