pragma solidity ^0.4.2;

contract Counter {
  uint public counter = 0;


  function count() {
    counter += 1;
  }
}
