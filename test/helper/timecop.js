var Timecop = {
  ff: function(blocks = 0){
    var height = web3.eth.blockNumber;
    if(typeof blocks !== 'number' || blocks === 0){
      return Promise.resolve(height);
    }

    console.log(height + '>>' + blocks);
    var counter = Counter.at(Counter.deployed_address);
    return counter.count().
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
