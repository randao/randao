contract('Randao', function(accounts) {

  it("web3.sha3 should return same as Solidity's sha3", function(done) {
    var zerostr = new Array(64).fill('0').join('');
    var str = web3.toHex('abc').substr(2);
    var longstr = (zerostr + str).substr(-64, 64);
    var result = web3.sha3('0x' + longstr);

    assert.equal('01749f991cfe31a6547a671af292c8f28b9ce9a6fcbd0b06b5b62e160799165d', result);
    done();
  });

});
