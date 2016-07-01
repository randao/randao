contract Randao {
  struct Participant {
    uint256   secret;
    bytes32   commitment;
    bool      reward;
  }

  struct Campaign {
    uint32    bnum;
    uint96    deposit;
    uint8     commitDeadline;
    uint8     commitBalkline;

    uint16    reveals;
    uint256   random;
    bool      settled;
    uint256   bountypot;
    uint32    commitNum;

    address[] paddresses;
    mapping (address => Participant) participants;
  }

  uint256 public numCampaigns;
  Campaign[] public campaigns;

  uint96 public callbackFee      = 100 finney;
  uint8  public constant version = 1;

  event CampaignAdded(uint campaignID, uint32 bnum, uint96 deposit, uint8 commitDeadline, uint8 commitBalkline);
  event Commit(uint CampaignId, address from, bytes32 commitment);

  function Randao() {}

  function newCampaign(uint32 _bnum, uint96 _deposit, uint8 _commitDeadline, uint8 _commitBalkline) returns (uint256 _campaignID) {
    if(block.number >= _bnum){ throw; }
    if(_commitDeadline <= 0){ throw; }
    if(_commitBalkline <= 0){ throw; }
    if(_commitDeadline >= _commitBalkline){ throw; }
    if(msg.value < 1 ether){ throw; }

    _campaignID = campaigns.length++;
    Campaign c = campaigns[_campaignID];
    numCampaigns++;
    c.bnum = _bnum;
    c.deposit = _deposit;
    c.commitDeadline = _commitDeadline;
    c.commitBalkline = _commitBalkline;
    c.bountypot = msg.value;

    CampaignAdded(_campaignID, _bnum, _deposit, _commitDeadline, _commitBalkline);
  }

  function commit(uint256 _campaignID, bytes32 _hs) external {
    Campaign c = campaigns[_campaignID];

    if(block.number >= c.bnum - c.commitBalkline && block.number < c.bnum - c.commitDeadline){

      if(_hs != "" && c.participants[msg.sender].commitment == ""){
        c.paddresses[c.paddresses.length++] = msg.sender;
        c.participants[msg.sender] = Participant(0, _hs, false);
        c.commitNum = c.commitNum + 1;
        Commit(_campaignID, msg.sender, _hs);
      } else {
        refund(msg.value);
      }
    } else {
      refund(msg.value);
    }
  }

  function reveal(uint256 _campaignID, uint256 _s) external {
    Campaign c = campaigns[_campaignID];

    uint256 rvalue;
    if(msg.value < c.deposit) {
      rvalue = msg.value;
    } else {
      rvalue = msg.value - c.deposit;
    }

    if(block.number < c.bnum && block.number >= c.bnum - c.commitDeadline){

      Participant p = c.participants[msg.sender];

      if(sha3(_s) == p.commitment){
        if(p.secret != _s){ c.reveals++; }
        p.secret = _s;
      }
    }
  }

  function getCommitment(uint256 _campaignID) external returns (bytes32) {
    Campaign c = campaigns[_campaignID];
    Participant p = c.participants[msg.sender];
    return p.commitment;
  }

  function checkSettled(uint256 _campaignID) returns (bool settled) {
    Campaign c = campaigns[_campaignID];
    if(block.number >= c.bnum) {
      if(!c.settled) { settle(c); }
    }
    return c.settled;
  }

  function getRandom(uint256 _campaignID) returns (uint256) {
    Campaign c = campaigns[_campaignID];

    if(block.number >= c.bnum) { // use campaign's random number
      if(!c.settled) { settle(c); }

      return c.random;
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
    }
  }

  function getMyBounty(uint256 _campaignID) external {
    Campaign c = campaigns[_campaignID];
    if(c.settled == true) {
      Participant p = c.participants[msg.sender];
      if(p.secret != 0 && p.reward == false){
        p.reward = true;
        var share = c.bountypot / c.reveals;
        if(!msg.sender.send(share)){ throw; }
      }
    }
    else {
      throw;
    }
  }

  function refund(uint rvalue) private {
    if(rvalue > txfee()){
      msg.sender.send(rvalue - txfee());
    }
  }

  function txfee() private returns (uint96) {
    return uint96(100 * tx.gasprice);
  }
}
