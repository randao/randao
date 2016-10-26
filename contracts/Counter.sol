pragma solidity ^0.4.3;

contract Counter {
  uint public counter = 0;


  function count() {
    counter += 1;
  }
}
