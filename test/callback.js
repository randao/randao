var Timecop = require('./helper/Timecop');
var utils = require('./helper/utils');

contract('Randao#callback', function(accounts) {

  it.only("other contract register a callback and send a bounty", function(done){
    var dice = Dice.at(Dice.deployed_address);
    var randao = Randao.at(Randao.deployed_address);
    var bnum = web3.eth.blockNumber + 12;

    dice.deposit({value: web3.toWei('10', 'ether'), from: accounts[0]})
    .then((rtn)=>{

      dice.randao(randao.address, bnum)
      .then((tx) => {

        randao.debug.call()
        .then((data) => {

          console.log(data);
          done();
        });

      });
    });
  });

  it("other contract get called after random number generated");
});
