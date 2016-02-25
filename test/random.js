var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao#random', function(accounts) {

  // TODO fix key
  it.only("generate random number if all revealed", function(done) {
    var deposit = web3.toWei('2', 'ether');
    var target_block = web3.eth.blockNumber + 12;
    var randao = Randao.at(Randao.deployed_address);

    randao.newCampaign.call(target_block, deposit, 6, 12)
    .then((campaignID) => {
      console.log('campaignID:', campaignID);
      var [randao, secrets, height, promise] = utils.prepare4reveals(accounts, campaignID, target_block);
      promise.then(() => {
        Timecop.ff(3).then(() => {
          Promise.all(secrets.map((secret, i) => { return randao.reveal(campaignID, secret, {from: accounts[i]}); }))
          .then(() => {

            Timecop.ff(7)
            .then(() => {
              randao.getRandom.call(campaignID)
              .then((random) => {
                var expected = secrets.reduce((pre, cur) => {return web3.toDecimal(pre) ^ web3.toDecimal(cur)});
                assert.equal(expected, random.toNumber());
                done();
              });
            })
          });
        })
      });
    })
  });

  it.skip("will not generate random number if anyone not reveal secret", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);
    var deposit = web3.toWei('2', 'ether');
    var key = web3.sha3(height, deposit, 6, 12);

    promise.then((result) => {

      // first participant will not reveal
      secrets.pop();

      Promise.all(secrets.map((secret, i) => { return randao.reveal(key, secret, {from: accounts[i]}); }))
      .then((result) => {

        Timecop.ff(3)
        .then(() => {

          randao.random.call(height, 6, 12)
          .then( (random) => {
            assert.equal(0, random.toNumber());
            done();
          });

        })
      });
    });
  });

  it.skip("participants will get commitment ethers back after generating random number", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);
    var deposit = web3.toWei('2', 'ether');
    var key = web3.sha3(height, deposit, 6, 12);

    promise.then((result) => {

      // first participant will not reveal
      secrets.pop();

      Promise.all(secrets.map((secret, i) => { return randao.reveal(key, secret, {from: accounts[i]}); }))
      .then((result) => {

        Timecop.ff(3)
        .then(() => {

          randao.random.call(height, 6, 12)
          .then( (random) => {
            assert.equal(0, random.toNumber());
            done();
          });

        })
      });
    });
  });

});
