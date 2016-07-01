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

    mapping (address => Participant) participants;
  }

  uint256 public numCampaigns;
  Campaign[] public campaigns;

  uint96 public callbackFee      = 100 finney;
  uint8  public constant version = 1;

  event CampaignAdded(uint256 campaignID, uint32 bnum, uint96 deposit, uint8 commitDeadline, uint8 commitBalkline);
  event Commit(uint256 CampaignId, address from, bytes32 commitment);
  event Reveal(uint256 CampaignId, address from, uint256 secret);

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
        c.participants[msg.sender] = Participant(0, _hs, false);
        c.commitNum = c.commitNum + 1;
        Commit(_campaignID, msg.sender, _hs);
      } else {
        throw;
      }
    } else {
      throw;
    }
  }

  function reveal(uint256 _campaignID, uint256 _s) external {
    Campaign c = campaigns[_campaignID];

    if(block.number < c.bnum && block.number >= c.bnum - c.commitDeadline){

      Participant p = c.participants[msg.sender];

      if(sha3(_s) == p.commitment){
        if(p.secret != _s){ c.reveals++; }
        p.secret = _s;
        c.random ^= p.secret;
        Reveal(_campaignID, msg.sender, _s);
      }
    }
  }

  function getCommitment(uint256 _campaignID) external returns (bytes32) {
    Campaign c = campaigns[_campaignID];
    Participant p = c.participants[msg.sender];
    return p.commitment;
  }

  function getRandom(uint256 _campaignID) returns (uint256) {
    Campaign c = campaigns[_campaignID];

    if(block.number >= c.bnum && c.reveals > 0) {
      if(!c.settled) { c.settled = true; }

      return c.random;
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
}
