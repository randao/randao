var Timecop = require('./helper/timecop');
var soliditySha3 = require('solidity-sha3');

contract('Randao', function(accounts) {
  it("randao campaign lifecycle", function(){
    var randao = Randao.deployed();
    var bnum = web3.eth.blockNumber + 20;
    var deposit = web3.toWei('10', 'ether');
    var campaignID;
    var secret;
    var commitment;

    console.log('target blockNumber: ', bnum);
    console.log('newCampaign at blockNumber: ', web3.eth.blockNumber);
    return randao.newCampaign(bnum, deposit, 12, 6, {from: accounts[0], value:web3.toWei(10, "ether")}).then((tx) => {
      return randao.numCampaigns.call();
    }).then(function(campaignID){
      assert.equal(campaignID.toNumber(), 1);

      return randao.follow(campaignID -1, { from: accounts[1], value: web3.toWei(10, "ether") });
    }).then(function(followed){
      secret = web3.toBigNumber('1234567899898989898989898298492849284928429482948294829482');
      console.log('secret:', secret.toString(10));
      var height = web3.eth.blockNumber + 10;
      commitment = soliditySha3.default(secret);
      console.log('commitment: ', commitment.toString(10));

      return Timecop.ff(9);
    }).then(() => {
      return randao.commit(campaignID - 1, commitment.toString(10), {value: web3.toWei('10', 'ether'), from: accounts[1]});
    }).then(() => {
      return randao.getCommitment.call(campaignID - 1, {from: accounts[1]});
    }).then((commit) => {
      assert.equal(commit, commitment);
      return Timecop.ff(7);
    }).then(() => {
      console.log('getMyBounty');
      return randao.getMyBounty(campaignID - 1, { from: accounts[1] });
    }).then(() => {
      console.log('refundBounty');
      return randao.refundBounty(campaignID - 1, { from: accounts[0] });
    }).then(() => {
    });
  });
});
