import 'Randao';

contract Dice {
  uint256 public random;
  address public addr;

  function Dice(uint8 _commitDeadline, uint8 _commitBalkline, uint96 _deposit, uint96 _callbackFee){
    addr = new Randao(_commitDeadline, _commitBalkline, _deposit, _callbackFee);
  }

  function randao(uint32 _bnum) returns (bool) {
    return addr.call.value(200 finney)(bytes4(sha3("random(uint32)")), _bnum, bytes4(sha3('callback(uint256)')));
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
