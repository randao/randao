contract('Randao', function(accounts) {
  it("refund if commit not happened in time window", function(done) {
    Randao.new().then(function(randao){
      var secret = '123456';
      var bnum = web3.eth.blockNumber + 19;

      var bal = web3.eth.getBalance(accounts[0]);
      randao.commit(bnum, web3.sha3(secret), {value: web3.toWei('12.3', 'ether')}).then(function(){
        var nbal = web3.eth.getBalance(accounts[0]);
        assert.equal(Math.round(parseFloat(web3.fromWei(nbal - bal, 'ether'))), 5);
        done();
      });

    }).catch(done);
  });

  it("refund if receive less than required deposit", function(done) {
    Randao.new().then(function(randao){
      var secret = '123456';
      var bnum = web3.eth.blockNumber + 9;

      var bal = web3.eth.getBalance(accounts[0]);
      randao.commit(bnum, web3.sha3(secret), {value: web3.toWei('12.3', 'ether')}).then(function(){
        var nbal = web3.eth.getBalance(accounts[0]);
        assert.equal(Math.round(parseFloat(web3.fromWei(bal - nbal, 'ether'))), 5);
        done();
      });

    }).catch(done);
  });

  it("refund if receive exceed required deposit", function(done) {
    Randao.new().then(function(randao){
      var secret = '123456';
      var bnum = web3.eth.blockNumber + 9;

      var bal = web3.eth.getBalance(accounts[0]);
      randao.commit(bnum, web3.sha3(secret), {value: web3.toWei('5.9', 'ether')}).then(function(){
        var nbal = web3.eth.getBalance(accounts[0]);
        assert.equal(Math.round(parseFloat(web3.fromWei(nbal - bal, 'ether'))), 5);
        done();
      });

    }).catch(done);
  });

});
