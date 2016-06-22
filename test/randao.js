var Timecop = require('./helper/timecop');

contract('Randao', function(accounts) {
  it("randao campaign lifecycle", function(done){
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber + 12;
    var deposit = web3.toWei('2', 'ether');

    console.log('target blockNumber: ', bnum);
    console.log('newCampaign at blockNumber: ', web3.eth.blockNumber);
    randao.newCampaign(bnum, deposit, 6, 12, {from: accounts[0],gas:100000,value:web3.toWei(10, "wei")})
      .then((tx) => {
      randao.numCampaigns.call().then(function(campaignID){
        assert.equal(campaignID.toNumber(), 1);

        var secret = web3.toHex('abcabc').slice(2);

        console.log('secret:', secret);
        var height = web3.eth.blockNumber + 10;
        var deposit = web3.toWei('2', 'ether');
        var commitment = '0x' + web3.sha3(secret, { encoding: 'hex' });
        console.log('commitment: ', commitment);

        var secret2 = web3.toHex('xxddd').slice(2);
        console.log('secret:', secret2);
        var commitment2 = '0x' + web3.sha3(secret2, { encoding: 'hex' });

        randao.commit(campaignID - 1, commitment, {value: web3.toWei('10', 'ether'), from: accounts[1]}).
        then(() => {
          randao.getCommitment.call(campaignID - 1, {from: accounts[1]}).
          then((commit) => {
            assert.equal(commit, commitment);
            randao.commit(campaignID - 1, commitment, {value: web3.toWei('10', 'ether'), from: accounts[2]}).
            then(() => {
              randao.getCommitment.call(campaignID - 1, {from: accounts[2]}).
              then((commit2) => {
                console.log("don't allow commit twice from one account");
                console.log('commit in contract: ', commit2);
                assert.equal(commit2, commitment);

                Timecop.ff(5).then(() => {
                  console.log('reveal at blockNumber: ', web3.eth.blockNumber);
                  randao.reveal(campaignID - 1, secret, {from: accounts[1]}).
                  then(() => {
                    Timecop.ff(5).then(() => {
                    randao.getRandom(campaignID - 1, {from: accounts[1]}).
                      then((random) => {
                        console.log('random: ', random);
                        done();
                      })
                    })
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
