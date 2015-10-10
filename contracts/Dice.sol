contract Dice {
  uint256 public random;

  function randao (address addr, uint32 bnum) returns (bool rtn) {
    return addr.call.value(200 finney)(bytes4(sha3("random(uint32)")), bnum, bytes4(sha3('callback(uint256)')));
  }

  function callback (uint256 r) {
    random = extractArg(msg.data);
  }

  function deposit() {
  }

  function extractArg(bytes data) returns (uint) {
    uint rtn = 0;
    for(uint i = 4; i < data.length; i++) {
      rtn += uint(data[i]) * (256 ** (data.length - 1 - i));
    }
    return rtn;
  }
}
