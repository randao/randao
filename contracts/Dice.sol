contract Dice {
  uint256 public random;

  function randao(address _addr, uint32 _bnum, uint96 _deposit, uint8 _commitDeadline, uint8 _commitBalkline) returns (bool) {
    return _addr.call.value(200 finney)(bytes4(sha3("random(uint32,uint96,uint8,uint8)")), _bnum, _deposit, _commitDeadline, _commitBalkline, bytes4(sha3('callback(uint256)')));
  }

  function randaoDefault(address _addr, uint32 _bnum) returns (bool) {
    return _addr.call.value(200 finney)(bytes4(sha3("random(uint32,uint96,uint8,uint8)")), _bnum, (10 ether), 6, 12, bytes4(sha3('callback(uint256)')));
  }

  function callback(uint256 _r) {
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
