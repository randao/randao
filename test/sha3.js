contract('Randao', function(accounts) {

  it("web3.sha3 should return same as Solidity's sha3", function(done) {
    var str = web3.toHex('abc').slice(2);
    var result = '0x' + web3.sha3(str, { encoding: 'hex' });

    assert.equal('0x4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45', result);
    done();
  });

});
