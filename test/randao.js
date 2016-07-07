var Timecop = require('./helper/timecop');

contract('Randao', function(accounts) {
  it("randao campaign lifecycle", function(done){
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber + 20;
    var deposit = web3.toWei('10', 'ether');

    console.log('target blockNumber: ', bnum);
    console.log('newCampaign at blockNumber: ', web3.eth.blockNumber);
    randao.newCampaign(bnum, deposit, 12, 6, {from: accounts[0], value:web3.toWei(10, "ether")})
      .then((tx) => {
      randao.numCampaigns.call().then(function(campaignID){
        assert.equal(campaignID.toNumber(), 1);

        randao.follow(campaignID -1, { from: accounts[1], value: web3.toWei(10, "ether") }).then(function(followed){
          var secret = web3.toHex('abcabc').slice(2);
          console.log('secret:', secret);
          var height = web3.eth.blockNumber + 10;
          var deposit = web3.toWei('2', 'ether');
          var commitment = '0x' + web3.sha3(secret, { encoding: 'hex' });
          console.log('commitment: ', commitment);

          Timecop.ff(9).then(() => {
            randao.commit(campaignID - 1, commitment, {value: web3.toWei('10', 'ether'), from: accounts[1]}).
            then(() => {
              randao.getCommitment.call(campaignID - 1, {from: accounts[1]}).
              then((commit) => {
                assert.equal(commit, commitment);
                Timecop.ff(5).then(() => {
                  console.log('reveal at blockNumber: ', web3.eth.blockNumber);
                  randao.reveal(campaignID - 1, secret, {from: accounts[1]}).
                  then(() => {
                    Timecop.ff(5).then(() => {
                    randao.getRandom(campaignID - 1, {from: accounts[1]}).
                      then((random) => {
                        console.log('random: ', random);

                        balance = web3.eth.getBalance(accounts[1]);
                        console.log(balance.plus(21).toString(10));
                        randao.getMyBounty(campaignID -1, { from: accounts[1] }).
                        then(() => {
                          newBalance = web3.eth.getBalance(accounts[1]);

                          console.log(newBalance.plus(21).toString(10));
                          done();
                        }) // getMyBounty

                      }) // gerRandom
                    })
                  }) // reveal
                })
              })
            }) // commit
          })
        }); // follow
      })
    }); // newCampaign
  });
});
