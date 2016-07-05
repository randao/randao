contract Randao {
  struct Participant {
      uint256   secret;
      bytes32   commitment;
      uint256   reward;
      bool      revealed;
      bool      rewarded;
  }

  struct Campaign {
      uint32    bnum;
      uint96    deposit;
      uint8     commitBalkline;
      uint8     commitDeadline;

      uint16    reveals;
      uint256   random;
      bool      settled;
      uint256   bountypot;
      uint32    commitNum;
      address   owner;

      mapping (address => Participant) participants;
  }

  uint256 public numCampaigns;
  Campaign[] public campaigns;
  uint256 public charityFund;
  address public founder;

  uint256 public bounty          = 1 ether;
  uint8  public constant version = 1;

  event CampaignAdded(uint256 campaignID, uint32 bnum, uint96 deposit, uint8 commitBalkline, uint8 commitDeadline);
  event Commit(uint256 CampaignId, address from, bytes32 commitment);
  event Reveal(uint256 CampaignId, address from, uint256 secret);

  modifier timeLineCheck(uint32 _bnum, uint8 _commitBalkline, uint8 _commitDeadline) {
      if (block.number >= _bnum) throw;
      if (_commitBalkline <= 0) throw;
      if (_commitDeadline <= 0) throw;
      if (_commitDeadline >= _commitBalkline) throw;
      if (block.number >= _bnum - _commitBalkline) throw;
      _
  }

  // Prevents methods from perfoming any value transfer
  modifier noEther() { if (msg.value > 0) throw; _}

  modifier checkBounty { if (msg.value < bounty) throw; _}

  modifier onlyFounder { if (founder != msg.sender) throw; _}

  modifier checkFund { if (charityFund == 0) throw; _}

  function Randao() {
      founder = msg.sender;
  }

  function newCampaign(
      uint32 _bnum,
      uint96 _deposit,
      uint8 _commitBalkline,
      uint8 _commitDeadline
  ) timeLineCheck(_bnum, _commitBalkline, _commitDeadline)
    checkBounty external returns (uint256 _campaignID) {
      _campaignID = campaigns.length++;
      Campaign c = campaigns[_campaignID];
      numCampaigns++;
      c.bnum = _bnum;
      c.owner = msg.sender;
      c.deposit = _deposit;
      c.commitBalkline = _commitBalkline;
      c.commitDeadline = _commitDeadline;
      c.bountypot = msg.value;

      CampaignAdded(_campaignID, _bnum, _deposit, _commitBalkline, _commitDeadline);
  }

  function commit(uint256 _campaignID, bytes32 _hs) external {
      Campaign c = campaigns[_campaignID];
      if (msg.value < c.deposit) throw;

      if (block.number >= c.bnum - c.commitBalkline
          && block.number < c.bnum - c.commitDeadline){
          if (_hs != "" && c.participants[msg.sender].commitment == ""){
              c.participants[msg.sender] = Participant(0, _hs, 0, false, false);
              c.commitNum = c.commitNum + 1;
              Commit(_campaignID, msg.sender, _hs);
          } else {
              throw;
          }
      } else {
          throw;
      }
  }

  function getCommitment(uint256 _campaignID) noEther external returns (bytes32) {
      Campaign c = campaigns[_campaignID];
      Participant p = c.participants[msg.sender];
      return p.commitment;
  }

  function reveal(uint256 _campaignID, uint256 _s) noEther external {
      Campaign c = campaigns[_campaignID];
      if (block.number < c.bnum
          && block.number >= c.bnum - c.commitDeadline) {
          Participant p = c.participants[msg.sender];
          if (sha3(_s) == p.commitment && !p.revealed) {
              c.reveals++;
              p.secret = _s;
              p.revealed = true;
              c.random ^= p.secret;
              Reveal(_campaignID, msg.sender, _s);
        }
      }
  }

  function getRandom(uint256 _campaignID) noEther external returns (uint256) {
      Campaign c = campaigns[_campaignID];
      if (block.number >= c.bnum && c.reveals > 0) {
          if (!c.settled) { c.settled = true; }
          charityFund += (c.commitNum - c.reveals) * c.deposit;
          return c.random;
      }
  }

  function getMyBounty(uint256 _campaignID) noEther external {
      Campaign c = campaigns[_campaignID];
      if (c.settled == true) {
          Participant p = c.participants[msg.sender];
          if (p.revealed && !p.rewarded) {
              uint256 share = c.bountypot / c.reveals;
              p.reward = share;
              p.rewarded = true;
              if (!msg.sender.send(share + c.deposit)) {
                  p.reward = 0;
                  p.rewarded = false;
              }
          }
      }
  }

  function refundBounty(uint256 _campaignID) noEther external {
      Campaign c = campaigns[_campaignID];
      if (block.number >= c.bnum
          && c.owner == msg.sender
          && c.reveals == 0
          && c.bountypot > 0) {
          uint256 bountypot = c.bountypot;
          c.bountypot = 0;
          if (!msg.sender.send(bountypot)) {
              c.bountypot = bountypot;
          }
      }
  }

  function withdrawFund() onlyFounder checkFund noEther external {
      uint256 fund = charityFund;
      charityFund = 0;
      if (!msg.sender.send(fund)) {
          charityFund = fund;
      }
  }
}
