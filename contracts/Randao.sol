pragma solidity ^0.4.3;

// version 1.0
contract Randao {
  struct Participant {
      uint256   secret;
      bytes32   commitment;
      uint256   reward;
      bool      revealed;
      bool      rewarded;
  }

  struct Consumer {
    address caddr;
    uint256 bountypot;
  }

  struct Campaign {
      uint32    bnum;               // Randao ends here - reveals complete and number can be calculated
      uint96    deposit;
      uint16    commitBalkline;     // commits can start this many blocks before bnum
      uint16    commitDeadline;     // commits end this many blocks after balkline (so this is the length of the commit stage)

      uint256   random;
      bool      settled;
      uint256   bountypot;
      uint32    commitNum;
      uint32    revealsNum;

      mapping (address => Consumer) consumers;
      mapping (address => Participant) participants;
  }

  uint256 public numCampaigns;
  Campaign[] public campaigns;
  address public founder;

  // Prevents methods from perfoming any value transfer
  modifier noEther() { if (msg.value > 0) throw; _; }

  modifier blankAddress(address _n) { if (_n != 0) throw; _; }

  modifier moreThanZero(uint256 _deposit) { if (_deposit <= 0) throw; _; }

  modifier notBeBlank(bytes32 _s) { if (_s == "") throw; _; }

  modifier beBlank(bytes32 _s) { if (_s != "") throw; _; }

  modifier beFalse(bool _t) { if (_t) throw; _; }

  function Randao() {
      founder = msg.sender;
  }

  event LogCampaignAdded(uint256 indexed campaignID,
                         address indexed from,
                         uint32 indexed bnum,
                         uint96 deposit,
                         uint16 commitBalkline,
                         uint16 commitDeadline,
                         uint256 bountypot);

  modifier timeLineCheck(uint32 _bnum, uint16 _commitBalkline, uint16 _commitDeadline) {
      if (block.number >= _bnum) throw;
      if (_commitBalkline <= 0) throw;
      if (_commitDeadline <= 0) throw;
      if (_commitDeadline >= _commitBalkline) throw;
      if (block.number >= _bnum - _commitBalkline) throw;
      _;
  }

  function newCampaign(
      uint32 _bnum,
      uint96 _deposit,
      uint16 _commitBalkline,
      uint16 _commitDeadline
  ) payable
    timeLineCheck(_bnum, _commitBalkline, _commitDeadline)
    moreThanZero(_deposit) external returns (uint256 _campaignID) {
      _campaignID = campaigns.length++;
      Campaign c = campaigns[_campaignID];
      numCampaigns++;
      c.bnum = _bnum;
      c.deposit = _deposit;
      c.commitBalkline = _commitBalkline;
      c.commitDeadline = _commitDeadline;
      c.bountypot = msg.value;
      c.consumers[msg.sender] = Consumer(msg.sender, msg.value);
      LogCampaignAdded(_campaignID, msg.sender, _bnum, _deposit, _commitBalkline, _commitDeadline, msg.value);
  }

  event LogFollow(uint256 indexed CampaignId, address indexed from, uint256 bountypot);

  function follow(uint256 _campaignID)
    external payable returns (bool) {
      Campaign c = campaigns[_campaignID];
      Consumer consumer = c.consumers[msg.sender];
      return followCampaign(_campaignID, c, consumer);
  }

  modifier checkFollowPhase(uint256 _bnum, uint16 _commitDeadline) {
      if (block.number > _bnum - _commitDeadline) throw;
      _;
  }

  function followCampaign(
      uint256 _campaignID,
      Campaign storage c,
      Consumer storage consumer
  ) checkFollowPhase(c.bnum, c.commitDeadline)
    blankAddress(consumer.caddr) internal returns (bool) {
      c.bountypot += msg.value;
      c.consumers[msg.sender] = Consumer(msg.sender, msg.value);
      LogFollow(_campaignID, msg.sender, msg.value);
      return true;
  }

  event LogCommit(uint256 indexed CampaignId, address indexed from, bytes32 commitment);

  function commit(uint256 _campaignID, bytes32 _hs) notBeBlank(_hs) external payable {
      Campaign c = campaigns[_campaignID];
      commitmentCampaign(_campaignID, _hs, c);
  }

  modifier checkDeposit(uint256 _deposit) { if (msg.value != _deposit) throw; _; }

  modifier checkCommitPhase(uint256 _bnum, uint16 _commitBalkline, uint16 _commitDeadline) {
      if (block.number < _bnum - _commitBalkline) throw;
      if (block.number > _bnum - _commitDeadline) throw;
      _;
  }

  function commitmentCampaign(
      uint256 _campaignID,
      bytes32 _hs,
      Campaign storage c
  ) checkDeposit(c.deposit)
    checkCommitPhase(c.bnum, c.commitBalkline, c.commitDeadline)
    beBlank(c.participants[msg.sender].commitment) internal {
      c.participants[msg.sender] = Participant(0, _hs, 0, false, false);
      c.commitNum++;
      LogCommit(_campaignID, msg.sender, _hs);
  }

  // For test
  function getCommitment(uint256 _campaignID) noEther external constant returns (bytes32) {
      Campaign c = campaigns[_campaignID];
      Participant p = c.participants[msg.sender];
      return p.commitment;
  }

  function shaCommit(uint256 _s) returns (bytes32) {
      return sha3(_s);
  }

  event LogReveal(uint256 indexed CampaignId, address indexed from, uint256 secret);

  function reveal(uint256 _campaignID, uint256 _s) noEther external {
      Campaign c = campaigns[_campaignID];
      Participant p = c.participants[msg.sender];
      revealCampaign(_campaignID, _s, c, p);
  }

  modifier checkRevealPhase(uint256 _bnum, uint16 _commitDeadline) {
      if (block.number <= _bnum - _commitDeadline) throw;
      if (block.number >= _bnum) throw;
      _;
  }

  modifier checkSecret(uint256 _s, bytes32 _commitment) {
      if (sha3(_s) != _commitment) throw;
      _;
  }

  function revealCampaign(
    uint256 _campaignID,
    uint256 _s,
    Campaign storage c,
    Participant storage p
  ) checkRevealPhase(c.bnum, c.commitDeadline)
    checkSecret(_s, p.commitment)
    beFalse(p.revealed) internal {
      p.secret = _s;
      p.revealed = true;
      c.revealsNum++;
      c.random ^= p.secret;
      LogReveal(_campaignID, msg.sender, _s);
  }

  modifier bountyPhase(uint256 _bnum){ if (block.number < _bnum) throw; _; }

  function getRandom(uint256 _campaignID) noEther external returns (uint256) {
      Campaign c = campaigns[_campaignID];
      return returnRandom(c);
  }

  function returnRandom(Campaign storage c) bountyPhase(c.bnum) internal returns (uint256) {
      if (c.revealsNum == c.commitNum) {
          c.settled = true;
          return c.random;
      }
  }

  // The commiter get his bounty and deposit, there are three situations
  // 1. Campaign succeeds.Every revealer gets his deposit and the bounty.
  // 2. Someone revels, but some does not,Campaign fails.
  // The revealer can get the deposit and the fines are distributed.
  // 3. Nobody reveals, Campaign fails.Every commiter can get his deposit.
  function getMyBounty(uint256 _campaignID) noEther external {
      Campaign c = campaigns[_campaignID];
      Participant p = c.participants[msg.sender];
      transferBounty(c, p);
  }

  function transferBounty(
      Campaign storage c,
      Participant storage p
    ) bountyPhase(c.bnum)
      beFalse(p.rewarded) internal {
      if (c.revealsNum > 0) {
          if (p.revealed) {
              uint256 share = calculateShare(c);
              returnReward(share, c, p);
          }
      // Nobody reveals
      } else {
          returnReward(0, c, p);
      }
  }

  function calculateShare(Campaign c) internal returns (uint256 _share) {
      // Someone does not reveal. Campaign fails.
      if (c.commitNum > c.revealsNum) {
          _share = fines(c) / c.revealsNum;
      // Campaign succeeds.
      } else {
          _share = c.bountypot / c.revealsNum;
      }
  }

  function returnReward(
      uint256 _share,
      Campaign storage c,
      Participant storage p
  ) internal {
      p.reward = _share;
      p.rewarded = true;
      if (!msg.sender.send(_share + c.deposit)) {
          p.reward = 0;
          p.rewarded = false;
      }
  }

  function fines(Campaign c) internal returns (uint256) {
      return (c.commitNum - c.revealsNum) * c.deposit;
  }

  // If the campaign fails, the consumers can get back the bounty.
  function refundBounty(uint256 _campaignID) noEther external {
      Campaign c = campaigns[_campaignID];
      returnBounty(_campaignID, c);
  }

  modifier campaignFailed(uint32 _commitNum, uint32 _revealsNum) {
      if (_commitNum == _revealsNum && _commitNum != 0) throw;
      _;
  }

  modifier beConsumer(address _caddr) {
      if (_caddr != msg.sender) throw;
      _;
  }

  function returnBounty(uint256 _campaignID, Campaign storage c)
    bountyPhase(c.bnum)
    campaignFailed(c.commitNum, c.revealsNum)
    beConsumer(c.consumers[msg.sender].caddr) internal {
      uint256 bountypot = c.consumers[msg.sender].bountypot;
      c.consumers[msg.sender].bountypot = 0;
      if (!msg.sender.send(bountypot)) {
          c.consumers[msg.sender].bountypot = bountypot;
      }
  }
}
