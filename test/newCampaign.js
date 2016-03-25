var utils = require('./helper/utils');
var Timecop = require('./helper/timecop');

contract('Randao', function(accounts) {

  it("new randao campaign", function(done){
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber + 12;
    var deposit = web3.toWei('2', 'ether');

    randao.newCampaign(bnum, deposit, 6, 12, {from: accounts[0], value: web3.toWei(10, "wei")}).
    then((tx) => {
      randao.numCampaigns.call().then(function(campaignID){
        assert.equal(campaignID.toNumber(), 1);
        done();
      })
    })
  })

  it("should create in correct time window", function(done){
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber;
    var deposit = web3.toWei('2', 'ether');

    randao.newCampaign(bnum, deposit, 6, 12, {from: accounts[0], value: web3.toWei(10, "wei")}).
    then(function(data){},function(data){ done(); })
  })

  it("check params validation", function(done){
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber;
    var deposit = web3.toWei('2', 'ether');

    randao.newCampaign(bnum, deposit, 6, 6, { from: accounts[0], value: web3.toWei(10, "wei") }).
    then(function(data){}, function(data){ done(); })
  })
})
