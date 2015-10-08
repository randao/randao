var Timecop = require('./helper/timecop');

contract('Timecop fast forward', function(accounts) {

  it("fast forward 3 blocks", (done) => {
    var current = web3.eth.blockNumber;

    Timecop.ff(3).then((height)=>{
      assert.equal(height - current, 3);
      done();
    });
  });

  it("fast forward zero block", (done) => {
    var current = web3.eth.blockNumber;

    Timecop.ff().then((height)=>{
      assert.equal(height - current, 0);
      done();
    });
  });
});
