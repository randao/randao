var Timecop = require('./helper/Timecop');

contract('Randao#commit', function(accounts) {
  it("add commitment", function(done){
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber + 12;
    var deposit = web3.toWei('2', 'ether');

    console.log('target blockNumber: ', bnum);
    console.log('newCampaign at blockNumber: ', web3.eth.blockNumber);
    randao.newCampaign(bnum, deposit, 6, 12, {from: accounts[0],gas:150000,value:web3.toWei(10, "ether")})
      .then((tx) => {
      randao.numCampaigns.call().then(function(campaignID){
        console.log('campaignID: ', campaignID.toNumber());

        var secret = web3.toHex('abcabc').slice(2);
        console.log('secret:', secret);
        var height = web3.eth.blockNumber + 10;
        var deposit = web3.toWei('2', 'ether');
        var commitment = '0x' + web3.sha3(secret, { encoding: 'hex' });
        console.log('commitment: ', commitment);
        randao.commit(campaignID - 1, commitment, {value: web3.toWei('10', 'ether'), from: accounts[1]}).
        then(() => {
          randao.getCommitment.call(campaignID - 1, {from: accounts[1]}).
          then((commit) => {
            console.log('commit in contract: ', commit);
            assert.equal(commit, commitment);
            done();
          })
        })
      })
    });
  })
});
