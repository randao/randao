contract('Randao', function(accounts) {
  it("should assert true", function(done) {
    var randao = Randao.at(Randao.deployed_address);
    assert.isTrue(true);
    done();
  });
});
