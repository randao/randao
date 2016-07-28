# Instruction

### New Campaign

Anyone who wants to generate random number, firstly need to create a campaign by calling `newCampaign` function.

```javascript
  function newCampaign(
      uint32 _bnum,
      uint96 _deposit,
      uint8 _commitBalkline,
      uint8 _commitDeadline
  )
```

This function has four parameters:
* `_bnum`: The target block number
* `_deposit`: The deposit for commiter
* `_commitBalkline`: The distance between block number of begining to commit and `_bnum`
* `_commitDeadline`: The distance between block number of ending to commit and `_bnum`

For instance, the current block number is 1840602, we need a random number at 1840900, and we wish the deposit to be 20 ethers, begin to commit at the block before 200 blocks of target block(i.e. start at 1840700, include 1840700), finish to commit at the block before 100 blocks of target block(i.e. end at 1840800, include 1840800).It's the reveal phase between 1840800(not include 1840800) and 1840900(not include 1840900).We can call the function as below:

`newCampaign(1840700, 20000000000000000000, 200, 100)`，and we can send some ethers as the bounty.

### Follow Campaign

The RANDAO demonder can follow a campaign by calling `Follow` function instead of creating a new campaign.

```javascript
function follow(uint256 _campaignID)
```

The `Follow` function has one parameter`_campaignID`.Anyone can find the detail infomation of the specified campaign.

The follow action must at the collecting phase or before it, otherwise it will fail.In previous example, the follow action must before the 1840800 block(include 1840800).The follower can also send some ethers for the bounty as well.

### Collecting valid sha3(s)

Anyone can commit random number to participate in the campaign by calling `commit` function:

```javascript
function commit(uint256 _campaignID, bytes32 _hs)
```

The `commit` function has two parameters:
* `_campaignID`
* `_hs`: The sha3 of random number.

Commiting the random number must send deposit, can not more or less than the deposit must be exactly equal to the deposit.Commiting must be in the collecting phase, otherwise it will fail.In previous example, the collecting phase is between 1840700 and 1840800.

### Reveal seed

```javascript
function reveal(uint256 _campaignID, uint256 _s)
```

`reveal` function has two parameters:

* `_campaignID`
* `_s`: sha3(s)

After the collecting phase, then it's reveal phase, every commiter then can reveal his seed, and the contract will verify the seed.If the seed is valid, it will be used to generate final random number.In previous example, the reveal phase is between 1840800 and 1840900.

### Fetch random number

```javascript
function getRandom(uint256 _campaignID)
```

Anyone can fetch the random number after the target block number.Only all the commiter succussfully reveal the seed, we take this round of campaign as valid.Otherwise, the campaign fails, and the contract confiscats the deposit of who did not reveal succussfully and distributes to the other participants.

### Get bounty and deposit

```javascript
function getMyBounty(uint256 _campaignID)
```

After the target block, the participants can get his deposit and bounty.

* If Campaign succeeds.Every revealer gets his deposit and the bounty.
* Someones revel succussfully,but some does not,Campaign fails.The revealer can get the deposit,and the fines are distributed to the honest ones.
* Nobody reveals,Campaign fails.Every commiter can get his deposit back.

### Refund bounty

```javascript
function refundBounty(uint256 _campaignID)
```

If the campaign fails, the campaign owner and the followers can get the bounty back by calling `refundBounty` function.
