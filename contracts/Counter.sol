pragma solidity ^0.5.0;

contract Counter {
    uint public counter = 0;


    function count() public {
        counter += 1;
    }
}
