var abi = require('ethereumjs-abi');

contract('Sha3', function(accounts) {
  it("sha3", function(){
    var sha3 = Sha3.deployed();

    var value = web3.toBigNumber('123456789');
    console.log('value: ', value.toString(10));
    var soliditySHA3 = abi.soliditySHA3(["uint"], [value.toNumber()]).toString('hex');
    console.log('web3 sha3: ', soliditySHA3);
    sha3.commit.call(value.toString(10), {from: accounts[0]}).then((v) => {
      console.log('solidity sha3: ', v);
      assert.equal(v, soliditySHA3,"Js sha3 match solidity sha3");
    })
  })
});
