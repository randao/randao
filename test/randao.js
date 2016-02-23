contract('Randao', function(accounts) {
  it("refund if commit not happened in time window", (done) => {
    var randao = Randao.at(Randao.deployed_address);
    var secret = '123456';
    var height = web3.eth.blockNumber + 10;
    var bnum = web3.eth.blockNumber + 19;
    var deposit = web3.toWei('2', 'ether');
    var key = web3.sha3(height, deposit, 6, 12);

    randao.random(height, deposit, 6, 12).then(() => {
      var bal = web3.eth.getBalance(accounts[0]);

      randao.commit(key, web3.sha3(secret), {value: web3.toWei('12.3', 'ether')}).then(() => {
        var nbal = web3.eth.getBalance(accounts[0]);
        assert.equal(Math.round(parseFloat(web3.fromWei(nbal - bal, 'ether'))), 5);
        done();
      });
    });
  });

  it("refund if receive less than required deposit", function(done) {
    var randao = Randao.at(Randao.deployed_address);
    var secret = '123456';
    var height = web3.eth.blockNumber + 10;
    var bnum = web3.eth.blockNumber + 19;
    var deposit = web3.toWei('20', 'ether');
    var key = web3.sha3(height, deposit, 6, 12);

    randao.random(height, deposit, 6, 12).then(() => {
      var bal = web3.eth.getBalance(accounts[0]);

      randao.commit(key, web3.sha3(secret), {value: web3.toWei('12.3', 'ether')}).then(() => {
        var nbal = web3.eth.getBalance(accounts[0]);
        assert.equal(Math.round(parseFloat(web3.fromWei(nbal - bal, 'ether'))), 5);
        done();
      });
    });
  });

  it("refund if receive exceed required deposit", function(done) {
    var randao = Randao.at(Randao.deployed_address);
    var secret = '123456';
    var height = web3.eth.blockNumber + 10;
    var bnum = web3.eth.blockNumber + 19;
    var deposit = web3.toWei('2', 'ether');
    var key = web3.sha3(height, deposit, 6, 12);

    randao.random(height, deposit, 6, 12).then(() => {
      var bal = web3.eth.getBalance(accounts[0]);

      randao.commit(key, web3.sha3(secret), {value: web3.toWei('12.3', 'ether')}).then(() => {
        var nbal = web3.eth.getBalance(accounts[0]);
        assert.equal(Math.round(parseFloat(web3.fromWei(nbal - bal, 'ether'))), 5);
        done();
      });
    });
  });

});
