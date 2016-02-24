var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao#reveal', function(accounts) {

  it("increase campaign's reveals count");

  it("reveal other account secret and replace other's participant address");

  // TODO: fix key
  it.skip("with correct reveals count", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);
    var deposit = web3.toWei('2', 'ether');
    var key = web3.sha3(height, deposit, 6, 12);

    promise.then((result) => {

      secrets.pop();
      secrets.shift();

      Promise.all(secrets.map((secret, i) => { return randao.reveal(key, secret, {from: accounts[i+1]}); }))
      .then((result) => {

        Timecop.ff(3)
        .then(() => {

          randao.reveals.call(key)
          .then( (count) => {
            assert.equal(count.toNumber(), 2);
            done();
          });

        })
      });
    });
  });

});
