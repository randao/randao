var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao', function(accounts) {

  it("randao contract holds commitment ethers", function(done) {
    var [randao, secrets, height, promise] = utils.prepare4reveals(accounts);

    promise.then((result) => {
      var bln = web3.eth.getBalance(randao.address);
      assert.equal(web3.fromWei(bln, 'ether').toNumber(), 40);
      done();
    });
  });

});
