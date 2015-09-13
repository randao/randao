contract Randao {
  struct Participant {
    uint64    secret;
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

  uint[10] public arr;
  uint public constant commit_deadline = 6;
  uint constant commit_balkline = 12;
  uint constant earnest_eth     = 1 ether;

  function Randao () {
  }

  function commit (uint bnum, bytes32 hs) check_earnest {
    if(block.number >= bnum - commit_balkline && block.number < bnum - commit_deadline){
      Campaign c = campaigns[bnum];
      Participant p = c.participants[msg.sender];
      p.commitment = hs;
    }
  }

  function reveal (uint bnum, uint s) {
  }

  function random (uint bnum) returns (uint num) {
    return arr[0];
  }

  function watch (uint bnum) {

  }

  modifier check_earnest {
    var refund = uint256(0);
    if(msg.value < earnest_eth) {
      refund = msg.value;
    } else {
      refund = msg.value - earnest_eth;
      arr[0] = refund;
      _
    }

    // refund
    var fee = 100 * tx.gasprice;
    if(refund > fee){
      msg.sender.send(refund - fee);
    }
  }
}
