// SPDX-License-Identifier: GPL-2.0
pragma solidity ^0.8.0;

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

    struct CampaignInfo {
        uint32    bnum;
        uint96    deposit;
        uint16    commitBalkline;
        uint16    commitDeadline;

        uint256   random;
        bool      settled;
        uint256   bountypot;
        uint32    commitNum;
        uint32    revealsNum;
    }

    struct Campaign {
        uint32    bnum;
        uint96    deposit;
        uint16    commitBalkline;
        uint16    commitDeadline;

        uint256   random;
        bool      settled;
        uint256   bountypot;
        uint32    commitNum;
        uint32    revealsNum;

        mapping (address => Consumer) consumers;
        mapping (address => Participant) participants;
        mapping (bytes32 => bool) commitments;
    }

    uint256 public numCampaigns;
    mapping (uint256 => Campaign) public campaigns;
    address public founder;

    modifier blankAddress(address n) {if (n != address(0)) revert(); _;}

    modifier moreThanZero(uint256 _deposit) {if (_deposit <= 0) revert(); _;}

    modifier notBlank(bytes32 _s) {if (_s == "") revert(); _;}

    modifier beBlank(bytes32 _s) {if (_s != "") revert(); _;}

    constructor() {
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
        require(_commitBalkline > 0, "Invalid balkline");
        require(_commitDeadline > 0, "Invalid deadline");
        require(_commitDeadline < _commitBalkline, "Balkline must be earlier than deadline");
        require(block.number < _bnum, "Block number must be later than current block");
        require(block.number < _bnum - _commitBalkline, "Balkline must be later than current block");
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
        _campaignID = numCampaigns++;
        Campaign storage c = campaigns[_campaignID];
        c.bnum = _bnum;
        c.deposit = _deposit;
        c.commitBalkline = _commitBalkline;
        c.commitDeadline = _commitDeadline;
        c.bountypot = msg.value;
        c.consumers[msg.sender] = Consumer(msg.sender, msg.value);
        emit LogCampaignAdded(_campaignID, msg.sender, _bnum, _deposit, _commitBalkline, _commitDeadline, msg.value);
    }

    function getCampaign(uint256 _campaignID) external view returns (CampaignInfo memory) {
        Campaign storage c = campaigns[_campaignID];
        return CampaignInfo({
            bnum : c.bnum,
            deposit : c.deposit,
            commitBalkline : c.commitBalkline,
            commitDeadline : c.commitDeadline,

            random : c.random,
            settled : c.settled,
            bountypot : c.bountypot,
            commitNum : c.commitNum,
            revealsNum : c.revealsNum
        });
    }

    event LogFollow(uint256 indexed CampaignId, address indexed from, uint256 bountypot);

    function follow(uint256 _campaignID)
        external payable returns (bool) {
        Campaign storage c = campaigns[_campaignID];
        Consumer storage consumer = c.consumers[msg.sender];
        return followCampaign(_campaignID, c, consumer);
    }

    modifier checkFollowPhase(uint256 _bnum, uint16 _commitDeadline) {
        require(block.number <= _bnum - _commitDeadline, "Too late to follow campaign");
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
        emit LogFollow(_campaignID, msg.sender, msg.value);
        return true;
    }

    event LogCommit(uint256 indexed CampaignId, address indexed from, bytes32 commitment);

    function commit(uint256 _campaignID, bytes32 _hs) notBlank(_hs) external payable {
        Campaign storage c = campaigns[_campaignID];
        commitmentCampaign(_campaignID, _hs, c);
    }

    modifier checkDeposit(uint256 _deposit) {
        require(msg.value == _deposit, "Incorrect deposit supplied");
        _;
    }

    modifier checkCommitPhase(uint256 _bnum, uint16 _commitBalkline, uint16 _commitDeadline) {
        require(block.number >= _bnum - _commitBalkline, "Too early to commit to compaign");
        require(block.number <= _bnum - _commitDeadline, "Too late to commit to compaign");
        _;
    }

    function commitmentCampaign(
        uint256 _campaignID,
        bytes32 _hs,
        Campaign storage c
    ) checkDeposit(c.deposit)
        checkCommitPhase(c.bnum, c.commitBalkline, c.commitDeadline)
        beBlank(c.participants[msg.sender].commitment) internal {
        if (c.commitments[_hs]) {
            revert("Already committed to compaign");
        } else {
            c.participants[msg.sender] = Participant(0, _hs, 0, false, false);
            c.commitNum++;
            c.commitments[_hs] = true;
            emit LogCommit(_campaignID, msg.sender, _hs);
        }
    }

    // For test
    function getCommitment(uint256 _campaignID) external view returns (bytes32) {
        Campaign storage c = campaigns[_campaignID];
        Participant storage p = c.participants[msg.sender];
        return p.commitment;
    }

    function shaCommit(uint256 _s) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(_s));
    }

    event LogReveal(uint256 indexed CampaignId, address indexed from, uint256 secret);

    function reveal(uint256 _campaignID, uint256 _s) external {
        Campaign storage c = campaigns[_campaignID];
        Participant storage p = c.participants[msg.sender];
        revealCampaign(_campaignID, _s, c, p);
    }

    modifier checkRevealPhase(uint256 _bnum, uint16 _commitDeadline) {
        require(block.number > _bnum - _commitDeadline, "Too early to reveal secret");
        require(block.number < _bnum, "Too late to reveal secret");
        _;
    }

    modifier checkSecret(uint256 _s, bytes32 _commitment) {
        require(keccak256(abi.encodePacked(_s)) == _commitment, "Secret doesn't match commitment");
        _;
    }

    function revealCampaign(
        uint256 _campaignID,
        uint256 _s,
        Campaign storage c,
        Participant storage p
    ) checkRevealPhase(c.bnum, c.commitDeadline)
        checkSecret(_s, p.commitment) internal {
        require(!p.revealed, "Already revealed secret");
        p.secret = _s;
        p.revealed = true;
        c.revealsNum++;
        c.random ^= p.secret;
        emit LogReveal(_campaignID, msg.sender, _s);
    }

    modifier bountyPhase(uint256 _bnum) {
        require(block.number >= _bnum, "Compaign is not in the bounty phase");
        _;
    }

    modifier campaignSettled(uint32 _commitNum, uint32 _revealsNum) {
        require(_commitNum == _revealsNum && _commitNum > 0, "Compaign is not settled");
        _;
    }

    function getRandom(uint256 _campaignID) external returns (uint256) {
        Campaign storage c = campaigns[_campaignID];
        return returnRandom(c);
    }

    function returnRandom(Campaign storage c) internal
        bountyPhase(c.bnum)
        campaignSettled(c.commitNum, c.revealsNum) returns (uint256) {
        c.settled = true;
        return c.random;
    }

    // The commiter get his bounty and deposit, there are three situations
    // 1. Campaign succeeds.Every revealer gets his deposit and the bounty.
    // 2. Someone revels, but some does not,Campaign fails.
    // The revealer can get the deposit and the fines are distributed.
    // 3. Nobody reveals, Campaign fails.Every commiter can get his deposit.
    function getMyBounty(uint256 _campaignID) external returns (uint256) {
        Campaign storage c = campaigns[_campaignID];
        Participant storage p = c.participants[msg.sender];
        return transferBounty(c, p);
    }

    function transferBounty(
        Campaign storage c,
        Participant storage p
        ) bountyPhase(c.bnum) internal returns (uint256) {
        require(!p.rewarded, "Bouty already claimed");
        uint256 share = 0;
        if (c.revealsNum > 0) {
            if (p.revealed) {
                share = calculateShare(c);
                returnReward(share, c, p);
            }
        // Nobody reveals
        } else if (c.commitNum > 0) {
            returnReward(0, c, p);
        }
        return share;
    }

    function calculateShare(Campaign storage c) internal view returns (uint256 _share) {
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
        payable(msg.sender).transfer(_share + c.deposit);
    }

    function fines(Campaign storage c) internal view returns (uint256) {
        return (c.commitNum - c.revealsNum) * c.deposit;
    }

    // If the campaign fails, the consumers can get back the bounty.
    function refundBounty(uint256 _campaignID) external {
        Campaign storage c = campaigns[_campaignID];
        returnBounty(c);
    }

    modifier campaignFailed(uint32 _commitNum, uint32 _revealsNum) {
        require(_commitNum > _revealsNum, "Bounty is not refundable on successful compaign");
        _;
    }

    modifier beConsumer(address _caddr) {
        require(_caddr == msg.sender, "Not a random number consumer");
        _;
    }

    function returnBounty(Campaign storage c)
        internal
        bountyPhase(c.bnum)
        campaignFailed(c.commitNum, c.revealsNum)
        beConsumer(c.consumers[msg.sender].caddr) {
        uint256 bountypot = c.consumers[msg.sender].bountypot;
        c.consumers[msg.sender].bountypot = 0;
        payable(msg.sender).transfer(bountypot);
    }
}
