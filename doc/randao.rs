//=====================================================================================================================
Consumer:

The consumer contract (on-chain) calls 3 methods on Randao contract

// start a new campaign
- function newCampaign(uint32 _bnum, uint96 _deposit, uint16 _commitBalkline, uint16 _commitDeadline) payable
            timeLineCheck(_bnum, _commitBalkline, _commitDeadline)
            moreThanZero(_deposit) external returns (uint256 _campaignID)

// follow an on-going campaign
- function follow(uint256 _campaignID) external payable returns (bool)


// refund bounty if campaign fails
- function refundBounty(uint256 _campaignID) external



//=====================================================================================================================

Participant:

The participant (off-chain service) calls 3 methods on Randao contract

// Get campaign
- function getCampaign(uint256 _campaignID) external view returns (CampaignInfo memory)

// Commit phase: rommit to a campaign
- function commit(uint256 _campaignID, bytes32 _hs) notBlank(_hs) external payable


// Reveal phase: reveal secret
- function reveal(uint256 _campaignID, uint256 _s) external
    - function shaCommit(uint256 _s) public pure returns (bytes32)


// Bounty phase: refund deposit and claim bounty
- function getMyBounty(uint256 _campaignID) external

