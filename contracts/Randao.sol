contract Randao {
  struct Participant {
    uint256   secret;
    bytes32   commitment;
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
  struct Consumer {
    address caddr;
    bytes  cbname;
  }

  //mapping (uint => uint) public numbers;
  mapping (uint32 => Campaign) public campaigns;

  uint8  constant commit_deadline = 6;
  uint8  constant commit_balkline = 12;
  uint96 constant deposit         = 10 ether;
  uint96 constant callback_fee    = 100 finney;
  uint8  public   version         = 1;

  function Randao () {
  }

  function commit (uint32 bnum, bytes32 hs) external check_deposit {
    if(block.number >= bnum - commit_balkline && block.number < bnum - commit_deadline){
      Campaign c = campaigns[bnum];

      c.paddresses[c.paddresses.length++] = msg.sender;
      Participant p = c.participants[msg.sender];
      p.commitment = hs;
    } else {
      refund(msg.value);
    }
  }

  function reveal (uint32 bnum, uint256 s) external {
    if(block.number < bnum && block.number >= bnum - commit_deadline){
      Campaign c = campaigns[bnum];

      Participant p = c.participants[msg.sender];

      if(sha3(s) == p.commitment){
        if(p.secret != s){ c.reveals++; }
        p.secret = s;
      }
    } else {
      refund(msg.value);
    }
  }

  function reveals (uint32 bnum) returns (uint r){
    return campaigns[bnum].reveals;
  }

  function test() returns (bytes32 rtn) {
    return sha3(0x00, 0x00, 0x0002);
  }

  function random (uint32 bnum) returns (uint num) {
    Campaign c = campaigns[bnum];

    if(block.number >= bnum) { // use campaign's random number
      if(c.settled) {
        return c.random;
      } else {
        settle(c);
        return c.random;
      }
    } else { // register random number callback
      // TODO: msg.sender or tx.origin ?
      if(msg.value >= callback_fee) {
        add2callback(c);
        return 1;
      } else {
        refund(msg.value);
        return 0;
      }
    }
  }

  function calculate(Campaign storage c) private {
    for (uint i = 0; i < c.paddresses.length; i++) {
      c.random ^= c.participants[c.paddresses[i]].secret;
    }
  }

  function settle(Campaign storage c) private {
    c.settled = true;

    if(c.reveals > 0){
      if(c.reveals == c.paddresses.length) calculate(c);

      if(c.random > 0) callback(c);

      refund_bounty(c);
    }
  }

  function refund_bounty(Campaign storage c) private {
    var fee = 100 * tx.gasprice;
    var share = c.bountypot / c.reveals;

    for (uint i = 0; i < c.paddresses.length; i++) {
      c.paddresses[i].send(share - txfee());
    }
  }

  function add2callback(Campaign storage c) private {
    c.consumers[c.consumers.length++] = Consumer(msg.sender, msg.data);
    c.bountypot += uint96(msg.value - txfee());
  }

  function callback(Campaign storage c) private {
    for (uint i = 0; i < c.consumers.length; i++) {
      var consumer = c.consumers[i];
      consumer.caddr.call(consumer.cbname, c.random);
    }
  }

  function refund (uint rvalue) private {
    // TODO: msg.sender or tx.origin ?
    if(rvalue > txfee()){
      msg.sender.send(rvalue - txfee());
    }
  }

  function txfee () private returns (uint96 fee) {
    return uint96(100 * tx.gasprice);
  }

  modifier check_deposit {
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
