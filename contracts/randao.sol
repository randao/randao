contract Randao {
  struct Participant {
    uint    secret;
    bytes32   commitment;
  }
  struct Campaign {
    mapping (address => Participant) participants;
    uint reveals;
  }
  struct Consumer {
    address addr;
  }

  mapping (uint => uint) public numbers;
  mapping (uint => Campaign) public campaigns;

  uint constant commit_deadline = 6;
  uint constant commit_balkline = 12;
  uint constant earnest_eth     = 1 ether;

  function commit (uint bnum, bytes32 hs) check_earnest returns (bool success) {
    if(block.number >= bnum - commit_balkline && block.number < bnum - commit_deadline){
      Campaign c = campaigns[bnum];
      Participant p = c.participants[msg.sender];
      p.commitment = hs;

      return true;
    } else {
      return false;
    }
  }

  function reveal (uint bunm, uint s) {
  }

  function random (uint bnum) {
  }

  function watch (uint bnum) {

  }

  modifier check_earnest {
    var refund = uint256(0);
    if(msg.value < earnest_eth) {
      refund = msg.value;
    } else {
      refund = msg.value - earnest_eth;
      _
    }

    // refund
    var fee = 100 * tx.gasprice;
    if(refund > fee){
      msg.sender.send(refund - fee);
    }
  }
}
