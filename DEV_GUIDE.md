#requirements

truffle ~> 5.0.0
ganache-cli

#development

Run `ganache-cli` with plenty of funds for gas:

```
ganache-cli -g 0 -l 8000000 -e 1000000000000
```

Run the test suite:

```
truffle test --network development
```

check contracts in directory `contracts/`
