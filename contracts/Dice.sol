contract Dice {
  uint256 public random;

  function randao (address addr, uint32 bnum) returns (bool rtn) {
    return addr.call.value(200 finney)(bytes4(sha3("random(uint32)")), bnum, 'callback');
  }

  function callback (uint r) {
    random = r;
  }

  function deposit() {
  }
}
