contract('Randao', function(accounts) {
  it("create a campaign", function(done) {
    var secret = '123456';
    var randao = Randao.at(Randao.deployed_address);
    console.log(randao.commit.value)
    randao.commit.call(1000, web3.sha3(secret)).then(function(result){
      assert.equal(result, true);
    }).catch(done);
  });
});
