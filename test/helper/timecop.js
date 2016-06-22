var Timecop = {
  ff: function(blocks){
    var height = web3.eth.blockNumber;
    if(typeof blocks !== 'number' || blocks === 0){
      return Promise.resolve(height);
    }

    console.log(height + '>>' + blocks);
    var counter = Counter.at(Counter.deployed_address);
    return counter.count({from: web3.eth.accounts[1]}).
      then(()=>{
        if(blocks > 1){
          return this.ff(blocks - 1);
        } else{
          return Promise.resolve(height + 1);
        }
      });
  }
}

module.exports = Timecop;
