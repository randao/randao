var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao', function(accounts) {

  it.only("with correct reveals count", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);
    promise.then((result) => {

      secrets.pop();
      secrets.shift();

      Promise.all(secrets.map((secret, i) => { return randao.reveal(height, secret, {from: accounts[i+1]}); }))
      .then((result) => {

        Timecop.ff(3)
        .then(() => {

          randao.reveals.call(height)
          .then( (count) => {
            assert.equal(count.toNumber(), 2);
            done();
          });

        })
      });
    });
  });

  xit("test", function(done) {
    var randao = Randao.at(Randao.deployed_address);

    randao.test.call().then((result) => {
      console.log(result);
      done();
    });
  });

});
