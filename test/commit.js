var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao#commit', function(accounts) {

  it.only("add commitment", function(done){
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber + 12;
    var deposit = web3.toWei('2', 'ether');

    randao.newCampaign(bnum, deposit, 6, 12, {from: accounts[0], value: web3.toWei(10, "wei")}).
    then((tx) => {
      randao.numCampaigns.call().then(function(campaignID){
        assert.equal(campaignID.toNumber(), 1);

        var zerostr = Array.apply(null, Array(3)).map(String.prototype.valueOf, "0").join('');
        var s = '5';
        var secret = '0x' + (zerostr + web3.toHex(s).substr(2)).substr(-64, 64);

        var commitment = web3.sha3(secret, true);
        console.log('commitment: ', commitment);

        Timecop.ff(4).then(() => {
          randao.commit.sendTransaction(campaignID - 1, commitment, {value: deposit, from: web3.eth.accounts[2]}).
          then((tx) => {
            console.log('commit plz wait...');
            console.log('commit at blockNumber: ', web3.eth.blockNumber);

            // TODO: should return the correct commitment
            randao.getCommitment.call(campaignID - 1, {from: web3.eth.accounts[2]}).
            then((r) => {
              console.log('get commitment', r);
              done();
            })
          });
        })
      })
    })
  })

  it("should commit in time window");

  it("don't allow commit twice from one account");

  it("randao contract holds commitment ethers");

});
