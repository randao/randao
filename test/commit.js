var Timecop = require('./helper/Timecop');

contract('Randao', function(accounts) {
  it("web3.sha3 should return same as Solidity's sha3", function(done) {
    var zerostr = new Array(64).fill('0').join('');
    var str = web3.toHex('abc').substr(2);
    var longstr = (zerostr + str).substr(-64, 64);
    var result = web3.sha3('0x' + longstr);

    assert.equal('01749f991cfe31a6547a671af292c8f28b9ce9a6fcbd0b06b5b62e160799165d', result);
    done();
  });

  it("generate random number", function(done) {
    var randao = Randao.at(Randao.deployed_address);
    var height = web3.eth.blockNumber + 10;
    var zerostr = new Array(64).fill('0').join('');

    var secrets = ['a', 'bc', 'def', 'g???'].map((s) => { return '0x' + (zerostr + web3.toHex(s).substr(2)).substr(-64, 64); });
    var commitments = secrets.map((s) => { return '0x' + web3.sha3(s); });

    console.log('RNG for block height:' + height);

    Promise.all(commitments.map((commitment, i) => { return randao.commit(height, commitment, {value: web3.toWei('10', 'ether'), from: accounts[i]}); }))
    .then((result) => {
      Promise.all(secrets.map((secret, i) => { return randao.reveal(height, secret, {from: accounts[i]}); }))
      .then((result) => {
        Timecop.ff(3)
        .then(() => {
          randao.random.call(height)
          .then((random) => {
            console.log('Current block height is: ' + web3.eth.blockNumber);
            var expected = secrets.reduce((pre, cur) => {return web3.toDecimal(pre) ^ web3.toDecimal(cur)});
            assert.equal(expected, random.toNumber());
            done();
          });
        })
      });
    });

  });
});
