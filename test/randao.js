contract('Randao', function(accounts) {
  it("create a campaign", function(done) {
    Randao.new().then(function(randao){
      var secret = '123456';
      var bnum = 1000;
      randao.commit(bnum, web3.sha3(secret), {value: web3.toWei('1.3', 'ether')}).then(function(){
        randao.commit_deadline().then(function(result){
          assert.equal(result, 6);
          done();
        }).catch(done);
      });
    }).catch(done);
  });

  it("refund if receive less than required deposit or exceed required deposit", function(done) {
    Randao.new().then(function(randao){
      var secret = '123456';
      var bnum = 1000;
      randao.commit(bnum, web3.sha3(secret), {value: web3.toWei('1.3', 'ether')}).then(function(){
        randao.random.call(1000).then(function(result){
          assert.equal(web3.fromWei(result.toNumber(), 'ether'), 0.3);
          done();
        }).catch(done);
      });
      randao.commit(bnum, web3.sha3(secret), {value: web3.toWei('0.9', 'ether')}).then(function(){
        randao.random.call(1000).then(function(result){
          assert.equal(web3.fromWei(result.toNumber(), 'ether'), 0.9);
          done();
        }).catch(done);
      });
    }).catch(done);
  });


});
