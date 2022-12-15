//=====================================================================================================================

Participant:

The participant service (off-chain) calls 3 methods on Randao contract

// Get campaign
- function getCampaign(uint256 _campaignID) external view returns (CampaignInfo memory)

// Commit phase: rommit to a campaign
- function commit(uint256 _campaignID, bytes32 _hs) notBlank(_hs) external payable


// Reveal phase: reveal secret
- function reveal(uint256 _campaignID, uint256 _s) external
    - function shaCommit(uint256 _s) public pure returns (bytes32)


// Bounty phase: refund deposit, claim bounty and reveal random number
- function getMyBounty(uint256 _campaignID) external
- function getRandom(uint256 _campaignID) external returns (uint256)


//=====================================================================================================================

Main thread:

Purpose: watch incoming campaigns on Randao contract.

    1> Load config.json file into memory.

    2> Initialize Prometheus[https://crates.io/crates/prometheus].

    3> Verify Findora blockchain info thru Web3 API.
        - Fetch actual chain info (chain ID, height etc.) and make sure chain ID matches config.json.
        - Fetch `numCampaigns` from Randao contract.
        - Print log (chain name, endpoint, chainId, numCampaigns, height) on success
        Example: "chain=Anvil... url=https://prod-testnet.prod.findora.org:8545, height=216584, numCampaigns=12, randao=0xA242C7682768c43B079b5B7dA09E2e7c80b1f5e2"

    4> Keep fetching `numCampaigns` on every new block.
        - Compare newly fetched `numCampaigns` against previous `pre_numCampaigns`
        Example: `numCampaigns > pre_numCampaigns` means that new campaign(s) had been created.

        - Keep watching until `numCampaigns > pre_numCampaigns`.
        - Fetch newly created campaign(s) using `getCampaign` method.
        - Make sure `config.maxDeposit >= campaign.deposit`.
        - Make sure `config.minRateOfReturn <= campaign.bountypot / campaign.deposit / (campaign.commitNum + 1)`.
        - Make sure `config.minGasReserve <= participant's pending FRA balance`
        - Make sure `bnum - commitBalkline > current height`
        - Make sure `commitDeadline > config.minRevealWindow`
        - Make sure `config.minRevealWindow > config.maxRevealDelay`
        - Make sure `config.maxCampaigns > current ongoing campaigns`
        - Start a new worker thread (with campaign info fetched from `getCampaign`) to participate in the campaign.


//=====================================================================================================================

Work thread:

Purpose: participate in one single campaign until the campaign completes.

    1> Keep compaign information (passed in by main thread) on hand.
    
    2> Commit to the campaign.
        - Generate a random secret u256 number `_s` and call `shaCommit` on Randao contract to calculate the commitment (_hs).
        - Call `commit` on Randao contract with proper `_campaignID` and `_hs` until get the transaction receipt.
        - End the work thread if the call fails and print proper log message.
            - Example error: "Commit failed, campaignID=12, err=VM Exception while processing transaction: Too late to commit to compaign"
        - Update (increment) prometheus metrics (e.g., "ONGOING_CAMPAIGNS" to track total number of ongoing campaigns)
        - Print commit information.
            Example info: "Commit succeed, campaignID=12, tx=0x3e8073efc8951034bcf6b0888be845983998a8898d541e9a58f57b09d77af806 gasPrice=10000000000"
    
    3> Reveal secret to the campaign.
        - Wait until `balkline` height (`campaign.bnum - campaign.commitBalkline`).
        - Randomly choose a height in range [balkline, balkline+config.minRevealDelay]
        - Call `reveal` on Randao contract with `_campaignID` and secret number `_s` and make sure that the call succeed.
        - End the work thread if the call fails (after block `campaign.bnum` and retry) and print proper log message.
            - Example error: "Reveal failed, campaignID=12, fines=3 err=VM Exception while processing transaction: Too late to reveal secret"
            - Update (decrement) prometheus metrics (e.g., "ONGOING_CAMPAIGNS" to track total number of ongoing campaigns)
        - Print reveal information.
            - Example info: "Reveal succeed, campaignID=12, tx=0x7d824b695d13ccf3007deb22eb9baddc6bef8105ccbfe5629ebb9d92cb1fe1b5 gasPrice=10000000000"
    
    4> Bounty claim.
        - Wait until height `campaign.bnum`.
            - Make eth-call to `getRandom` on Randao contract with `_campaignID` to get the random number.
            - If eth-call fails, print error information.
                - Example error: "Get random failed, campaignID=12, err=VM Exception while processing transaction: Compaign is not settled"
        - Call `getMyBounty` on Randao contract with `_campaignID` and make sure that the call succeed.
            - Print error message if it's necessary for debugging.
            - Example error: "Get bounty failed, campaignID=12, err=VM Exception while processing transaction: Compaign is not in the bounty phase"
        - Update (decrement) prometheus metrics (e.g., "ONGOING_CAMPAIGNS" to track total number of ongoing campaigns)
        - Print bounty information.
            Example info: "Bounty claimed, campaignID=12, bounty=1.5 tx=0xa31a1e4f6f2a089ca57323c6491c0aba4274a4c81be36cc74728cd9c1f16562b gasPrice=10000000000"


//=====================================================================================================================

More considerations:

    1> Crash protection.
        - work thread can save `_campaignID` and secret `_s` to file (e.g., campaign_12.json)
        - main thread can load above file to resume the campaign if it's still open.
    
    2> Nicely stop service.
        - main thread stop participating in new campaigns.
        - work thread keep handling their own campaigns until complete.
        - main thread exit until all work thread complete.

    3> Avoid fines
        - stops service whenever a work thread detects fines?