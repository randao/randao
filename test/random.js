var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao#random', function(accounts) {

  it("generate random number if all revealed", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);
    promise.then((result) => {

      Promise.all(secrets.map((secret, i) => { return randao.reveal(height, secret, {from: accounts[i]}); }))
      .then((result) => {

        Timecop.ff(3)
        .then(() => {

          randao.random.call(height, 6, 12)
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
      secrets.pop();

      Promise.all(secrets.map((secret, i) => { return randao.reveal(height, secret, {from: accounts[i]}); }))
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

  it("participants will get commitment ethers back after generating random number", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);
    promise.then((result) => {

      // first participant will not reveal
      secrets.pop();

      Promise.all(secrets.map((secret, i) => { return randao.reveal(height, secret, {from: accounts[i]}); }))
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
