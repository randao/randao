var Timecop = require('./helper/timecop');
var soliditySha3 = require('solidity-sha3');

contract('Randao', function(accounts) {
  it("randao campaign lifecycle", function() {
    var randao = Randao.deployed();
    var bnum = web3.eth.blockNumber + 20;
    var deposit = web3.toWei('10', 'ether');
    var campaignID;
    var commitment;
    var secret;

    console.log('target blockNumber: ', bnum);
    console.log('newCampaign at blockNumber: ', web3.eth.blockNumber);
    return randao.newCampaign(bnum, deposit, 12, 6, {from: accounts[0], value:web3.toWei(10, "ether")}).then((tx) => {
      return randao.numCampaigns.call();
    }).then(function(campaignID){
      assert.equal(campaignID.toNumber(), 1);

      return randao.follow(campaignID -1, { from: accounts[1], value: web3.toWei(10, "ether") });
    }).then(function(followed){
      secret = web3.toBigNumber('334567899898989898989898298492849284928429482948294829482');
      console.log('secret:', secret.toString(10));
      var height = web3.eth.blockNumber + 10;
      commitment = soliditySha3.default(secret);
      console.log('commitment: ', commitment.toString(10));

      return Timecop.ff(9);
    }).then(() => {
      console.log('commit', commitment);
      console.log(web3.eth.getBalance(accounts[1]));
      return randao.commit(campaignID - 1, commitment, {value: web3.toWei('10', 'ether'), from: accounts[1]})
    }).then(() => {
      return randao.getCommitment.call(campaignID - 1, {from: accounts[1]});
    }).then((commit) => {
      assert.equal(commit, commitment);
      return Timecop.ff(5);
    }).then(() => {
      console.log('reveal at blockNumber: ', web3.eth.blockNumber);
      return randao.reveal(campaignID - 1, secret.toString(10), {from: accounts[1]});
    }).then(() => {
      return Timecop.ff(5);
    }).then(() => {
      console.log('getRandom');
      return randao.getRandom.call(campaignID - 1, {from: accounts[1]});
    }).then((random) => {
      console.log('random: ', random.toNumber());
      return randao.getRandom(campaignID - 1, {from: accounts[1]});
    }).then((tx) => {
      return randao.getMyBounty(campaignID -1, { from: accounts[1] });
    }).then(() => {
    });
  });
});
