var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao#commit', function(accounts) {

  it("should commit in time window");

  it("don't allow commit twice from one account");

  it("don't allow change commitment");

  it("randao contract holds commitment ethers", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);

    promise.then((result) => {
      var bln = web3.eth.getBalance(randao.address);
      // TODO: Need fix
      assert.equal(bln.toNumber(), 400);
      done();
    });
  });

});
