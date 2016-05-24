var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao#callback', function(accounts) {

  it("other contract register a callback and send a bounty", function(done){
    var dice = Dice.at(Dice.deployed_address);
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber + 12;
    var deposit = web3.toWei('2', 'ether');

    dice.deposit({value: web3.toWei('10', 'ether'), from: accounts[0]})
    .then((rtn)=>{
      console.log('current blockNumber: ', web3.eth.blockNumber);
      console.log('bnum: ', bnum);
      dice.randao(randao.address, bnum, deposit, 6, 12)
      .then((tx) => {
        console.log('randao at: ', web3.eth.blockNumber);
        randao.numCampaigns.call().then(function(campaignID){
          console.log('campaignID: ', campaignID.toNumber());
          var [secrets, height, promise] = utils.prepare4reveals(randao, accounts, campaignID - 1);
          var key = web3.sha3(height, deposit, 6, 12);

          var s = '5';
          var zerostr = new Array(64).fill('0').join('');
          console.log('x');
          var secret = '0x' + (zerostr + web3.toHex(s).substr(2)).substr(-64, 64);
          console.log('xx');
          var height = web3.eth.blockNumber + 10;
          var deposit = web3.toWei('2', 'ether');
          var commitment = web3.sha3(secret, true);
          console.log('accounts[0]', accounts[0]);
          var rpro = randao.commit(campaignID - 1, commitment, {value: web3.toWei('10', 'ether'), from: accounts[0]})
          console.log('accounts[0]', accounts[0]);
          rpro.then(() => {
            console.log('x');
            done();
          })

          console.log('test');


          // promise.then(() => {
          //   console.log('commit at: ', web3.eth.blockNumber);
          //   Timecop.ff(accounts, 4).then(() => {
          //     Promise.all(secrets.map((secret, i) => { return randao.reveal(key, secret, {from: accounts[i]}); }))
          //     .then(() => {
          //       console.log('reveal at: ', web3.eth.blockNumber);
          //       Timecop.ff(2)
          //       .then(() => {
          //         randao.random(height, deposit, 6, 12)
          //         .then(() => {

          //           randao.random.call(height, deposit, 6, 12)
          //           .then((random) => {

          //             dice.random.call()
          //             .then((dicerandom) => {
          //               assert.equal(random.toNumber(), dicerandom.toNumber());
          //               done();
          //             });
          //           });
          //         });
          //       })
          //     });
          //   });
          // })
        })
      });
    });
  });

  it("other contract get called after random number generated");
});
