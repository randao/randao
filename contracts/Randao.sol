contract Randao {
  struct Participant {
    uint256   secret;
    bytes32   commitment;
  }
  struct Consumer {
    address caddr;
    bytes  cbname;
  }
  struct Campaign {
    address[] paddresses;
    uint16    reveals;

    uint256   random;
    bool      settled;
    uint96    bountypot;

    Consumer[] consumers;

    mapping (address => Participant) participants;
  }

  mapping (uint32 => Campaign) public campaigns;

  uint8  public commitDeadline;
  uint8  public commitBalkline;
  uint96 public deposit;
  uint96 public callbackFee;
  uint8  public constant version = 1;

  function Randao(uint8 _commitDeadline, uint8 _commitBalkline, uint96 _deposit, uint96 _callbackFee) {
    commitDeadline = _commitDeadline;
    commitBalkline = _commitBalkline;
    deposit = _deposit;
    callbackFee = _callbackFee;
  }

  function commit(uint32 _bnum, bytes32 _hs) external checkDeposit {
    if(block.number >= _bnum - commitBalkline && block.number < _bnum - commitDeadline){
      Campaign c = campaigns[_bnum];

      if(_hs != "" && c.participants[msg.sender].commitment == ""){
        c.paddresses[c.paddresses.length++] = msg.sender;
        c.participants[msg.sender] = Participant(0, _hs);
      } else { // can't change commitment after commited
        refund(msg.value);
      }
    } else {
      refund(msg.value);
    }
  }

  //TODO: allow reveal others secrets
  function reveal(uint32 _bnum, uint256 _s) external {
    if(block.number < _bnum && block.number >= _bnum - commitDeadline){
      Campaign c = campaigns[_bnum];

      Participant p = c.participants[msg.sender];

      if(sha3(_s) == p.commitment){
        if(p.secret != _s){ c.reveals++; }
        p.secret = _s;
      }
    }
  }

  function reveals(uint32 _bnum) returns (uint r){
    return campaigns[_bnum].reveals;
  }

  function test() returns (bool) {
    return true;
  }

  function random(uint32 _bnum) returns (uint) {
    Campaign c = campaigns[_bnum];

    if(block.number >= _bnum) { // use campaign's random number
      if(!c.settled) { settle(c); }

      return c.random;
    } else { // register random number callback
      // TODO: msg.sender or tx.origin ?
      if(msg.value >= callbackFee) {
        add2callback(c);
        return 1;
      } else {
        refund(msg.value);
        return 0;
      }
    }
  }

  function calculate(Campaign storage _c) private {
    for (uint i = 0; i < _c.paddresses.length; i++) {
      _c.random ^= _c.participants[_c.paddresses[i]].secret;
    }
  }

  function settle(Campaign storage _c) private {
    _c.settled = true;

    if(_c.reveals > 0){
      if(_c.reveals == _c.paddresses.length) calculate(_c);

      if(_c.random > 0) callback(_c);

      refundBounty(_c);
    }
  }

  function refundBounty(Campaign storage _c) private {
    var fee = 100 * tx.gasprice;
    var share = _c.bountypot / _c.reveals;

    for (uint i = 0; i < _c.paddresses.length; i++) {
      _c.paddresses[i].send(share - txfee());
    }
  }

  function add2callback(Campaign storage _c) private {
    _c.consumers[_c.consumers.length++] = Consumer(msg.sender, slice(msg.data, 36, 4));
    _c.bountypot += uint96(msg.value - txfee());
  }

  function callback(Campaign storage _c) private {
    for (uint i = 0; i < _c.consumers.length; i++) {
      var consumer = _c.consumers[i];
      consumer.caddr.call(consumer.cbname, _c.random);
    }
  }

  function refund(uint rvalue) private {
    // TODO: msg.sender or tx.origin ?
    if(rvalue > txfee()){
      msg.sender.send(rvalue - txfee());
    }
  }

  function txfee() private returns (uint96) {
    return uint96(100 * tx.gasprice);
  }

  function slice(bytes _str, uint _index, uint _size) returns (bytes) {
    uint rindex;
    bytes memory newstr;
    if(_size == 0 || _index + _size >= _str.length){
      rindex = _str.length;
    } else {
      rindex = _index + _size;
    }
    for(uint i=_index; i< rindex; i++) {
      newstr[i-_index] = _str[i];
    }
    return newstr;
  }

  modifier checkDeposit {
    var rvalue = uint256(0);
    if(msg.value < deposit) {
      rvalue = msg.value;
    } else {
      rvalue = msg.value - deposit;
      _
    }

    refund(rvalue);
  }
}
