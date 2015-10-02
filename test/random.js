var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao', function(accounts) {

  it("generate random number if all revealed", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);
    promise.then((result) => {

      Promise.all(secrets.map((secret, i) => { return randao.reveal(height, secret, {from: accounts[i]}); }))
      .then((result) => {

        Timecop.ff(3)
        .then(() => {

          randao.random.call(height)
          .then( (random) => {

            var expected = secrets.reduce((pre, cur) => {return web3.toDecimal(pre) ^ web3.toDecimal(cur)});
            assert.equal(expected, random.toNumber());
            done();
          });

        })
      });
    });
  });

  it("will not generate random number if anyone not reveal secret", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);
    promise.then((result) => {

      // first participant will not reveal
      secrets.shift();

      Promise.all(secrets.map((secret, i) => { return randao.reveal(height, secret, {from: accounts[i]}); }))
      .then((result) => {

        Timecop.ff(3)
        .then(() => {

          randao.random.call(height)
          .then( (random) => {
            assert.equal(0, random.toNumber());
            done();
          });

        })
      });
    });
  });

});
