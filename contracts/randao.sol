contract Randao {
  struct Participant {
    uint64    secret;
    bytes32   commitment;
  }
  struct Campaign {
    address[] paddresses;
    mapping (address => Participant) participants;
    uint reveals;
  }
  struct Consumer {
    address addr;
  }

  //mapping (uint => uint) public numbers;
  mapping (uint => Campaign) public campaigns;

  uint constant commit_deadline = 6;
  uint constant commit_balkline = 12;
  uint constant earnest_eth     = 10 ether;

  function Randao () {
  }

  function commit (uint bnum, bytes32 hs) external check_earnest {
    if(block.number >= bnum - commit_balkline && block.number < bnum - commit_deadline){
      Campaign c = campaigns[bnum];

      c.paddresses[c.paddresses.length++] = msg.sender;
      Participant p = c.participants[msg.sender];
      p.commitment = hs;
    } else {
      refund(msg.value);
    }
  }

  function reveal (uint bnum, uint64 s) external {
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

  function random (uint bnum) constant returns (uint num) {
    var random = uint(0);
    Campaign c = campaigns[bnum];
    if(block.number >= bnum && c.reveals > 0 && c.reveals == c.paddresses.length){
      for (uint i = 0; i < c.paddresses.length; i++) {
        random |= c.participants[c.paddresses[i]].secret;
      }
    }
    return random;
  }

  function version () returns (uint8 ver){
    return uint8(1);
  }

  function refund (uint rvalue) private {
    // refund
    var fee = 100 * tx.gasprice;
    if(rvalue > fee){
      msg.sender.send(rvalue - fee);
    }
  }

  modifier check_earnest {
    var rvalue = uint256(0);
    if(msg.value < earnest_eth) {
      rvalue = msg.value;
    } else {
      rvalue = msg.value - earnest_eth;
      _
    }

    refund(rvalue);
  }
}
