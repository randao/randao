var utils = require('./helper/utils');
var Timecop = require('./helper/timecop');

contract('Randao', function(accounts) {

  it.only("randao campaign lifecycle", function(done){
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber + 12;
    var deposit = web3.toWei('2', 'ether');

    console.log('newCampaign at blockNumber: ', web3.eth.blockNumber);
    randao.newCampaign(bnum, deposit, 6, 12, {from: accounts[0],gas:100000,value:web3.toWei(10, "wei")})
      .then((tx) => {
      randao.numCampaigns.call().then(function(campaignID){
        console.log('campaignID: ', campaignID.toNumber());

        var s = '5';
        var zerostr = new Array(64).fill('0').join('');
        var secret = '0x' + (zerostr + web3.toHex(s).substr(2)).substr(-64, 64);
        var height = web3.eth.blockNumber + 10;
        var deposit = web3.toWei('2', 'ether');
        var commitment = web3.sha3(secret, { encoding: 'hex' });
        console.log('commitment: ', commitment);
        randao.commit(campaignID - 1, commitment, {value: web3.toWei('10', 'ether'), from: accounts[1]}).
        then(() => {
          randao.getCommitment.call(campaignID - 1, {from: accounts[1]}).
          then((commit) => {
            console.log('commit in contract: ', commit);
            Timecop.ff(accounts, 4).then(() => {
              randao.reveal(campaignID - 1, secret, {from: accounts[1]}).
              then(() => {
                Timecop.ff(4).then(() => {
                randao.getRandom.call(campaignID - 1, {from: accounts[1]}).
                  then((random) => {
                    console.log('random: ', random.toNumber());
                    done();
                  })
                })
              })
            })
          })
        })
      })
    });
  });

  it("refund if commit not happened in time window", (done) => {
    var randao = Randao.at(Randao.deployed_address);
    var secret = '123456';
    var height = web3.eth.blockNumber + 10;
    var bnum = web3.eth.blockNumber + 19;
    var deposit = web3.toWei('2', 'ether');
    randao.newCampaign(height, deposit, 6, 12).then(() => {
      var bal = web3.eth.getBalance(accounts[0]);

      randao.commit(1, web3.sha3(secret), {value: web3.toWei('12.3', 'ether')}).then(() => {
        var nbal = web3.eth.getBalance(accounts[0]);
        assert.equal(Math.round(parseFloat(web3.fromWei(nbal - bal, 'ether'))), 0);
        done();
      });
    });
  });

  it("refund if receive less than required deposit", function(done) {
    var randao = Randao.at(Randao.deployed_address);
    var secret = '123456';
    var height = web3.eth.blockNumber + 10;
    var bnum = web3.eth.blockNumber + 19;
    var deposit = web3.toWei('20', 'ether');

    randao.newCampaign(height, deposit, 6, 12).then(() => {
      var bal = web3.eth.getBalance(accounts[0]);

      var nbal = web3.eth.getBalance(accounts[0]);
      assert.equal(Math.round(parseFloat(web3.fromWei(nbal - bal, 'ether'))), 0);
      done();
    });
  });

  it("refund if receive exceed required deposit", function(done) {
    var randao = Randao.at(Randao.deployed_address);
    var secret = '123456';
    var height = web3.eth.blockNumber + 10;
    var bnum = web3.eth.blockNumber + 19;
    var deposit = web3.toWei('2', 'ether');

    randao.newCampaign(height, deposit, 6, 12).then(() => {
      var bal = web3.eth.getBalance(accounts[0]);

      randao.commit(1, web3.sha3(secret), {value: web3.toWei('12.3', 'ether')}).then(() => {
        var nbal = web3.eth.getBalance(accounts[0]);
        assert.equal(Math.round(parseFloat(web3.fromWei(nbal - bal, 'ether'))), 0);
        done();
      });
    });
  });
});
